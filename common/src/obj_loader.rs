use std::{
  io::BufRead,
  path::{Path, PathBuf},
};

use image::RgbImage;

#[derive(Default)]
struct ObjLoader {
  base: PathBuf,
}

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

pub type VAIdx = [usize; 3];

pub struct Obj {
  pub v: Vec<[f32; 3]>,
  pub vn: Vec<[f32; 3]>,
  pub vt: Vec<[f32; 3]>,

  pub mtl_lib: MtlLib,
  pub groups: Vec<Group>,
}

impl Obj {
  pub fn load_from<P: AsRef<Path>>(path: &P) -> Result<Self> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let loader = ObjLoader::new(path);
    let obj = loader.parse(&mut reader)?;
    Ok(obj)
  }
}

#[derive(PartialEq)]
pub struct Group {
  pub name: String,
  pub f: Vec<Vec<VAIdx>>,
  pub usemtl: Option<String>,
}

impl Group {
  pub fn trigs(&self) -> impl Iterator<Item = [VAIdx; 3]> + '_ {
    self.f.iter().flat_map(|face| {
      let v0 = face[0];
      face
        .windows(2)
        .skip(1)
        .map(move |pair| [v0, pair[0], pair[1]])
    })
  }
}

impl Default for Group {
  fn default() -> Self {
    Group {
      name: "Default".to_string(),
      f: Vec::new(),
      usemtl: None,
    }
  }
}

#[allow(dead_code, non_snake_case)]
#[derive(PartialEq, Clone)]
pub struct Mtl {
  pub name: String,
  pub Ns: f32,
  pub Ni: f32,
  pub d: f32,
  pub Tr: f32,
  pub Tf: [f32; 3],
  pub illum: u32,
  pub Ka: [f32; 3],
  pub Kd: [f32; 3],
  pub Ks: [f32; 3],
  pub Ke: [f32; 3],
  pub map_Ka: Option<RgbImage>,
  pub map_Kd: Option<RgbImage>,
  pub map_Ks: Option<RgbImage>,
  pub map_bump: Option<RgbImage>,
}

impl Default for Mtl {
  fn default() -> Self {
    Mtl {
      name: "Default".to_string(),
      Ns: 0.0,
      Ni: 0.0,
      d: 0.0,
      Tr: 0.0,
      Tf: [0.0, 0.0, 0.0],
      illum: 0,
      Ka: [0.0, 0.0, 0.0],
      Kd: [0.0, 0.0, 0.0],
      Ks: [0.0, 0.0, 0.0],
      Ke: [0.0, 0.0, 0.0],
      map_Ka: None,
      map_Kd: None,
      map_Ks: None,
      map_bump: None,
    }
  }
}

#[derive(Default)]
pub struct MtlLib {
  pub mtls: Vec<Mtl>,
}

// immutable raw obj data
pub struct SimpleObj {
  pub v: Vec<[f32; 3]>,
  #[allow(dead_code)]
  pub vn: Vec<[f32; 3]>,
  #[allow(dead_code)]
  pub vt: Vec<[f32; 3]>,
  #[allow(dead_code)]
  pub g: String,
  #[allow(dead_code)]
  pub f: Vec<Vec<VAIdx>>,
}

impl SimpleObj {
  pub fn load_from<P: AsRef<Path>>(path: &P) -> Result<Self> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let loader = ObjLoader::new(path);
    let obj = loader.parse_simple(&mut reader)?;
    Ok(obj)
  }

  pub fn bounding_box(&self) -> [(f32, f32); 3] {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for v in self.v.iter() {
      for i in 0..3 {
        min[i] = min[i].min(v[i]);
        max[i] = max[i].max(v[i]);
      }
    }
    [(min[0], max[0]), (min[1], max[1]), (min[2], max[2])]
  }

  pub fn center(&self) -> [f32; 3] {
    let bbox = self.bounding_box();
    [
      (bbox[0].0 + bbox[0].1) / 2.0,
      (bbox[1].0 + bbox[1].1) / 2.0,
      (bbox[2].0 + bbox[2].1) / 2.0,
    ]
  }

  pub fn trigs(&self) -> impl Iterator<Item = [VAIdx; 3]> + '_ {
    self.f.iter().flat_map(|face| {
      let v0 = face[0];
      face
        .windows(2)
        .skip(1)
        .map(move |pair| [v0, pair[0], pair[1]])
    })
  }
}

impl ObjLoader {
  fn new<P: AsRef<Path>>(obj_path: P) -> Self {
    let obj_path = obj_path.as_ref().to_path_buf();
    let obj_base = obj_path.parent().unwrap().to_path_buf();
    Self { base: obj_base }
  }

  fn parse_vec3f(&self, line: &str) -> Result<[f32; 3]> {
    let mut parts = line.split_whitespace();
    let x = parts.next().ok_or("missing x")?.parse()?;
    let y = parts.next().ok_or("missing y")?.parse()?;
    let z = parts.next().ok_or("missing z")?.parse()?;
    Ok([x, y, z])
  }

  fn parse_v(&self, line: &str) -> Result<[f32; 3]> {
    self.parse_vec3f(line)
  }

  fn parse_vec2f_or_vec3f(&self, line: &str) -> Result<[f32; 3]> {
    let mut parts = line.split_whitespace();
    let x = parts.next().ok_or("missing x")?.parse()?;
    let y = parts.next().ok_or("missing y")?.parse()?;
    let z = parts.next().unwrap_or("0.0").parse()?;
    Ok([x, y, z])
  }

  fn parse_str(&self, line: &str) -> Result<String> {
    Ok(line.to_string())
  }

  fn parse_f_vertex(&self, vert: &str) -> Result<VAIdx> {
    let mut parts = vert.split('/');
    let v = parts.next().ok_or("missing v")?.parse()?;
    let vt = parts.next().ok_or("missing vt")?.parse()?;
    let vn = parts.next().ok_or("missing vn")?.parse()?;
    Ok([v, vt, vn])
  }

  fn parse_f(&self, line: &str) -> Result<Vec<VAIdx>> {
    let parts = line.split_whitespace();
    let mut vec = Vec::with_capacity(4);
    for part in parts {
      vec.push(self.parse_f_vertex(part)?);
    }
    Ok(vec)
  }

  fn parse_simple<R: BufRead>(&self, input: &mut R) -> Result<SimpleObj> {
    let mut v = Vec::new();
    let mut vn = Vec::new();
    let mut vt = Vec::new();
    let mut g = String::new();
    let mut f = Vec::new();

    for line in input.lines() {
      let line = line?;
      let line = line.trim();
      if line.is_empty() || line.starts_with('#') {
        continue;
      }

      let (ty, data) = line.split_once(' ').ok_or("missing type")?;
      let data = data.trim();
      match ty {
        "v" => v.push(self.parse_v(data)?),
        "vn" => vn.push(self.parse_vec3f(data)?),
        "vt" => vt.push(self.parse_vec2f_or_vec3f(data)?),
        "g" => g = self.parse_str(data)?,
        "f" => f.push(self.parse_f(data)?),
        _ => {}
      }
    }

    Ok(SimpleObj { v, vn, vt, g, f })
  }

  fn parse<R: BufRead>(&self, input: &mut R) -> Result<Obj> {
    let mut v = Vec::new();
    let mut vn = Vec::new();
    let mut vt = Vec::new();
    let mut groups = Vec::new();
    let mut current_group = Group::default();
    let mut mtl_lib = MtlLib::default();

    for line in input.lines() {
      let line = line?;
      let line = line.trim();
      if line.is_empty() || line.starts_with('#') {
        continue;
      }

      let (ty, data) = line.split_once(' ').ok_or("missing type")?;
      let data = data.trim();
      match ty {
        "mtllib" => {
          let path = self.base.join(data);
          let file = std::fs::File::open(path)?;
          let mut reader = std::io::BufReader::new(file);
          mtl_lib = self.parse_mtl_lib(&mut reader)?;
        }

        "v" => v.push(self.parse_v(data)?),
        "vn" => vn.push(self.parse_vec3f(data)?),
        "vt" => vt.push(self.parse_vec2f_or_vec3f(data)?),
        "g" => {
          if current_group != Group::default() {
            groups.push(current_group);
          }

          current_group = Group {
            name: data.to_string(),
            ..Group::default()
          };
        }
        "usemtl" => current_group.usemtl = Some(data.to_string()),
        "f" => current_group.f.push(self.parse_f(data)?),
        _ => {
          // println!("ignoring line: {}", line);
        }
      }
    }

    if current_group != Group::default() {
      groups.push(current_group);
    }

    Ok(Obj {
      v,
      vn,
      vt,
      mtl_lib,
      groups,
    })
  }

  pub fn parse_mtl_lib<R: BufRead>(&self, input: &mut R) -> Result<MtlLib> {
    let mut mtls = Vec::new();
    let mut current_mtl = Mtl::default();

    for line in input.lines() {
      let line = line?;
      let line = line.trim();
      if line.is_empty() || line.starts_with('#') {
        continue;
      }

      let (ty, data) = line.split_once(' ').ok_or("missing type")?;
      let data = data.trim();
      match ty {
        "newmtl" => {
          if current_mtl != Mtl::default() {
            mtls.push(current_mtl);
          }

          current_mtl = Mtl {
            name: data.to_string(),
            ..Mtl::default()
          };
        }
        "Ns" => current_mtl.Ns = data.parse()?,
        "Ni" => current_mtl.Ni = data.parse()?,
        "d" => current_mtl.d = data.parse()?,
        "Tr" => current_mtl.Tr = data.parse()?,
        "Tf" => current_mtl.Tf = self.parse_vec3f(data)?,
        "illum" => current_mtl.illum = data.parse()?,
        "Ka" => current_mtl.Ka = self.parse_vec3f(data)?,
        "Kd" => current_mtl.Kd = self.parse_vec3f(data)?,
        "Ks" => current_mtl.Ks = self.parse_vec3f(data)?,
        "Ke" => current_mtl.Ke = self.parse_vec3f(data)?,
        "map_Ka" => {
          let path = self.base.join(data);
          let img = image::open(path)?;
          current_mtl.map_Ka = Some(img.into_rgb8());
        }
        "map_Kd" => {
          let path = self.base.join(data);
          let img = image::open(path)?;
          current_mtl.map_Kd = Some(img.into_rgb8());
        }
        "map_Ks" => {
          let path = self.base.join(data);
          let img = image::open(path)?;
          current_mtl.map_Ks = Some(img.into_rgb8());
        }
        "map_bump" => {
          let path = self.base.join(data);
          let img = image::open(path)?;
          current_mtl.map_bump = Some(img.into_rgb8());
        }
        _ => {}
      }
    }

    if current_mtl != Mtl::default() {
      mtls.push(current_mtl);
    }

    Ok(MtlLib { mtls })
  }
}

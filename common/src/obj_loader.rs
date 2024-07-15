use std::{io::BufRead, path::Path};

#[derive(Default)]
struct ObjLoader;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

// immutable raw obj data
pub struct RawObj {
  pub v: Box<[[f32; 3]]>,
  #[allow(dead_code)]
  pub vn: Box<[[f32; 3]]>,
  #[allow(dead_code)]
  pub vt: Box<[[f32; 3]]>,
  #[allow(dead_code)]
  pub g: Box<str>,
  #[allow(dead_code)]
  pub f: Box<[Box<[[usize; 3]]>]>,
}

impl RawObj {
  pub fn load_from(path: &Path) -> Result<Self> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let loader = ObjLoader;
    let obj = loader.parse(&mut reader)?;
    Ok(obj)
  }
}

impl ObjLoader {
  fn parse_v(&self, line: &str) -> Result<[f32; 3]> {
    let mut parts = line.split_whitespace();
    let x = parts.next().ok_or("missing x")?.parse()?;
    let y = parts.next().ok_or("missing y")?.parse()?;
    let z = parts.next().ok_or("missing z")?.parse()?;
    Ok([x, y, z])
  }

  fn parse_vn(&self, line: &str) -> Result<[f32; 3]> {
    // just a shortcut for now
    self.parse_v(line)
  }

  fn parse_vt(&self, line: &str) -> Result<[f32; 3]> {
    // just a shortcut for now
    self.parse_v(line)
  }

  fn parse_g(&self, line: &str) -> Result<String> {
    Ok(line.to_string())
  }

  fn parse_f_vertex(&self, vert: &str) -> Result<[usize; 3]> {
    let mut parts = vert.split('/');
    let v = parts.next().ok_or("missing v")?.parse()?;
    let vt = parts.next().ok_or("missing vt")?.parse()?;
    let vn = parts.next().ok_or("missing vn")?.parse()?;
    Ok([v, vt, vn])
  }

  fn parse_f(&self, line: &str) -> Result<Box<[[usize; 3]]>> {
    let parts = line.split_whitespace();
    let mut vec = Vec::with_capacity(4);
    for part in parts {
      vec.push(self.parse_f_vertex(part)?);
    }
    Ok(vec.into_boxed_slice())
  }

  fn parse<R: BufRead>(&self, input: &mut R) -> Result<RawObj> {
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
        "vn" => vn.push(self.parse_vn(data)?),
        "vt" => vt.push(self.parse_vt(data)?),
        "g" => g = self.parse_g(data)?,
        "f" => f.push(self.parse_f(data)?),
        _ => {}
      }
    }

    Ok(RawObj {
      v: v.into_boxed_slice(),
      vn: vn.into_boxed_slice(),
      vt: vt.into_boxed_slice(),
      g: g.into_boxed_str(),
      f: f.into_boxed_slice(),
    })
  }
}

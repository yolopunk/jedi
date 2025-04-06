use sysinfo::LoadAvg;

pub(crate) trait ToLinuxLoadTuple {
  fn to_tuple(&self) -> (f64, f64, f64);
}

impl ToLinuxLoadTuple for LoadAvg {
  fn to_tuple(&self) -> (f64, f64, f64) {
    (self.one, self.five, self.fifteen)
  }
}

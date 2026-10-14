/// A single-key DAWG with unshared states for targeted binary corruption cases.
pub struct DawgFixture {
    pub units: Vec<u32>,
    pub guide: Vec<[u8; 2]>,
    pub path: Vec<usize>,
}

impl DawgFixture {
    pub fn new(key: &str, payload: &[u8]) -> Self {
        let labels: Vec<_> = key
            .bytes()
            .chain(std::iter::once(1))
            .chain(payload.iter().copied())
            .collect();
        let length = (labels.len() + 2) * 256;
        let mut units = vec![0; length];
        let mut guide = vec![[0, 0]; length];
        let mut path = vec![0];
        for (step, label) in labels.iter().enumerate() {
            let index = path[step];
            let base = (step + 1) * 256;
            let next = base ^ usize::from(*label);
            units[index] |= u32::try_from(index ^ base).unwrap() << 10;
            guide[index][0] = *label;
            units[next] = u32::from(*label);
            path.push(next);
        }
        let last = *path.last().unwrap();
        let value_index = (labels.len() + 1) * 256;
        units[last] |= (1 << 8) | (u32::try_from(last ^ value_index).unwrap() << 10);
        units[value_index] = 1 << 31;
        Self { units, guide, path }
    }

    pub fn bytes(&self) -> Vec<u8> {
        let mut result = u32::try_from(self.units.len())
            .unwrap()
            .to_le_bytes()
            .to_vec();
        result.extend(self.units.iter().flat_map(|n| n.to_le_bytes()));
        result.extend(u32::try_from(self.guide.len()).unwrap().to_le_bytes());
        result.extend(self.guide.iter().flatten());
        result
    }
}

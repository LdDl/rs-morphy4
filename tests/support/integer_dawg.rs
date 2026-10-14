pub struct IntegerFixture {
    pub bytes: Vec<u8>,
    pub terminal: usize,
    pub value_index: usize,
}

impl IntegerFixture {
    pub fn new(key: &str, value: u32) -> Self {
        let labels = key.as_bytes();
        let count = (labels.len() + 2) * 256;
        let mut units = vec![0u32; count];
        let mut guide = vec![[0u8; 2]; count];
        let mut index = 0;
        for (step, label) in labels.iter().enumerate() {
            let base = (step + 1) * 256;
            let next = base ^ usize::from(*label);
            units[index] |= u32::try_from(index ^ base).unwrap() << 10;
            guide[index][0] = *label;
            units[next] = u32::from(*label);
            index = next;
        }
        let value_index = (labels.len() + 1) * 256;
        units[index] |= (1 << 8) | (u32::try_from(index ^ value_index).unwrap() << 10);
        units[value_index] = (1 << 31) | value;
        let mut bytes = u32::try_from(count).unwrap().to_le_bytes().to_vec();
        bytes.extend(units.into_iter().flat_map(u32::to_le_bytes));
        bytes.extend(u32::try_from(count).unwrap().to_le_bytes());
        bytes.extend(guide.into_iter().flatten());
        Self {
            bytes,
            terminal: index,
            value_index,
        }
    }

    pub fn set_unit(&mut self, index: usize, value: u32) {
        let offset = 4 + index * 4;
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

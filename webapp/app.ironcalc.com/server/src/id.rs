use rand::{rngs::StdRng, Rng, SeedableRng};
const CHARS: [char; 64] = [
    '_', '!', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

fn random(size: usize) -> Vec<u8> {
    let mut rng = StdRng::from_entropy();
    let mut result: Vec<u8> = vec![0; size];

    rng.fill(&mut result[..]);

    result
}

pub fn new_id() -> String {
    let size = 15;
    let mask = CHARS.len() - 1;
    let step: usize = 5;
    let mut id = String::new();

    loop {
        let bytes = random(step);

        for &byte in &bytes {
            let byte = byte as usize & mask;

            id.push(CHARS[byte]);

            if id.len() >= size + 2 {
                return id;
            }
        }
        id.push('-');
    }
}

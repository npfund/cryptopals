use base64::decode as base64_decode;
use bit_vec::BitVec;
use hex::decode;
use hex::encode as hex_encode;
use openssl::symm::{decrypt, Cipher};
use std::cmp::Ordering;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

fn main() {
    let input = BufReader::new(File::open("8.txt").expect("File not found!"));
    let lines = input
        .lines()
        .map(|x| Vec::from(x.unwrap().as_bytes()))
        .collect::<Vec<Vec<u8>>>();

    for (i, line) in lines.iter().enumerate() {
        for (lc, left) in line.chunks(16).enumerate() {
            for (rc, right) in line.chunks(16).enumerate().skip(lc + 1) {
                if left == right {
                    println!("{}@{},{}: {}", i + 1, lc + 1, rc + 1, String::from_utf8(left.to_vec()).unwrap());
                }
            }
        }
    }
}

fn fixed_xor(input: &[u8], mask: &[u8]) -> Vec<u8> {
    return input
        .into_iter()
        .zip(mask.into_iter())
        .map(|(x, y)| x ^ y)
        .collect::<Vec<u8>>();
}

fn repeating_xor(input: &Vec<u8>, key: &Vec<u8>) -> Vec<u8> {
    return input
        .into_iter()
        .zip(key.iter().cycle().into_iter())
        .map(|(x, y)| x ^ y)
        .collect::<Vec<u8>>();
}

fn crack_single_byte_xor(input: &Vec<u8>) -> (u8, Vec<u8>) {
    let characters = (0..=255).collect::<Vec<u8>>();
    let mut strings: Vec<(u8, Vec<u8>)> = characters
        .iter()
        .map(|character| {
            let mask = vec![character.clone(); input.len()];
            return (character.clone(), fixed_xor(input, &mask));
        })
        .collect();

    strings.sort_unstable_by(|left, right| sort_by_score_desc(&left.1, &right.1));

    return strings.first().unwrap().clone();
}

fn crack_repeating_xor(input: &Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let key_size = (2..=40)
        .fold((0, 0.0), |size, k| {
            let chunks = input.chunks_exact(k * 2);
            let total_chunks = chunks.len() as f32;
            let distance = chunks.fold(0.0, |d, chunk| {
                d + hamming_distance(&chunk[0..k], &chunk[k..2 * k]) as f32 / k as f32
            }) / total_chunks;

            if size.1 == 0.0 || distance < size.1 {
                (k, distance)
            } else {
                size
            }
        })
        .0;

    let blocks: Vec<Vec<u8>> = input.chunks(key_size).map(|x| Vec::from(x)).collect();

    let mut transposed_blocks: Vec<Vec<u8>> = Vec::with_capacity(blocks.len());
    for _ in 0..key_size {
        transposed_blocks.push(Vec::new());
    }

    for position in 0..key_size {
        for block in &blocks {
            match block.get(position) {
                Some(byte) => transposed_blocks
                    .get_mut(position)
                    .unwrap()
                    .push(byte.clone()),
                None => {}
            }
        }
    }

    let mut key = Vec::new();
    let mut decoded_blocks = Vec::new();
    for block in transposed_blocks {
        let solution = crack_single_byte_xor(&block);
        key.push(solution.0);
        decoded_blocks.push(solution.1);
    }

    let mut final_bytes = Vec::new();
    for i in 0..decoded_blocks.get(0).unwrap().len() {
        for block in &decoded_blocks {
            match block.get(i) {
                Some(byte) => final_bytes.push(byte.clone()),
                None => {}
            }
        }
    }

    return (key, final_bytes);
}

fn decrypt_aes_128_ecb(key: &[u8], input: &[u8]) -> String {
    let cipher = Cipher::aes_128_ecb();

    String::from_utf8(decrypt(cipher, &key.as_ref(), None, &input).unwrap()).unwrap()
}

fn sort_by_score_desc(left: &Vec<u8>, right: &Vec<u8>) -> Ordering {
    let left_score = english_score(left);
    let right_score = english_score(right);

    return if left_score > right_score {
        Ordering::Less
    } else if left_score < right_score {
        Ordering::Greater
    } else {
        Ordering::Equal
    };
}

fn english_score(input: &Vec<u8>) -> i64 {
    let mut score = 0;

    for byte in input {
        if (*byte < 32 || *byte > 122) && *byte != 10 {
            score -= 1000;
        }

        if (*byte >= 65 && *byte <= 90) || (*byte >= 97 && *byte <= 122) {
            score += 1;
        }
    }

    return score;
}

fn hamming_distance(left: &[u8], right: &[u8]) -> u32 {
    let mut left_bits = BitVec::from_bytes(left);
    let mut right_bits = BitVec::from_bytes(right);
    let left_len = left_bits.len();
    let right_len = right_bits.len();

    if left_len > right_len {
        right_bits.grow(left_len - right_len, false);
    } else if right_len > left_len {
        left_bits.grow(right_len - left_len, false);
    }

    return left_bits
        .iter()
        .zip(right_bits.iter())
        .fold(0, |sum, (l, r)| sum + if l == r { 0 } else { 1 });
}

fn pkcs7(input: &str, block_size: usize) -> String {
    let padding = (block_size - input.len() % block_size) as u8;
    format!("{}{}", input, String::from(padding as char).repeat(padding as usize))
}

#[cfg(test)]
mod test {
    use super::*;
    use base64::encode as base64_encode;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;

    #[test]
    fn case1test() {
        let input = "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d";
        let expected = "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29t";

        assert_eq!(expected, base64_encode(&decode(input).unwrap()));
    }

    #[test]
    fn case2test() {
        let input = "1c0111001f010100061a024b53535009181c";
        let xor = "686974207468652062756c6c277320657965";
        let expected = "746865206b696420646f6e277420706c6179";

        assert_eq!(
            expected,
            hex_encode(&fixed_xor(&decode(input).unwrap(), &decode(xor).unwrap()))
        );
    }

    #[test]
    fn case3test() {
        let input = "1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736";
        let expected = "Cooking MC's like a pound of bacon";

        assert_eq!(
            expected,
            String::from_utf8(crack_single_byte_xor(&decode(input).unwrap()).1).unwrap()
        );
    }

    #[test]
    fn case4test() {
        let file = BufReader::new(File::open("4.txt").unwrap());
        let mut lines: Vec<(u8, Vec<u8>)> = file
            .lines()
            .filter_map(|x| x.ok())
            .map(|x| decode(x).unwrap())
            .map(|x| crack_single_byte_xor(&x))
            .collect();

        lines.sort_unstable_by(|left, right| sort_by_score_desc(&left.1, &right.1));
        assert_eq!(
            "Now that the party is jumping\n",
            String::from_utf8(lines.first().unwrap().clone().1).unwrap()
        );
    }

    #[test]
    fn case5test() {
        let key: Vec<u8> = "ICE".bytes().collect();
        let bytes: Vec<u8> = "Burning 'em, if you ain't quick and nimble
I go crazy when I hear a cymbal"
            .bytes()
            .collect();
        let xord = repeating_xor(&bytes, &key);

        assert_eq!("0b3637272a2b2e63622c2e69692a23693a2a3c6324202d623d63343c2a26226324272765272a282b2f20430a652e2c652a3124333a653e2b2027630c692b20283165286326302e27282f", hex_encode(&xord));
    }

    #[test]
    fn hamming_test() {
        let left: Vec<u8> = "this is a test".bytes().collect();
        let right: Vec<u8> = "wokka wokka!!!".bytes().collect();
        assert_eq!(37, hamming_distance(&left, &right));
    }

    #[test]
    fn case9test() {
        let input = "YELLOW SUBMARINE";
        assert_eq!("YELLOW SUBMARINE\x04\x04\x04\x04", pkcs7(input, 20))
    }
}

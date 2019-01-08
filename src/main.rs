extern crate hex;
extern crate base64;
extern crate bit_vec;

use hex::decode;
use hex::encode as hex_encode;
use base64::encode as base64_encode;
use base64::decode as base64_decode;
use std::cmp::Ordering;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Read;
use bit_vec::BitVec;

fn main() {
    let input = BufReader::new(File::open("6.txt").expect("File not found!"));
    let base64 = input.lines()
        .map(|x| x.unwrap())
        .collect::<Vec<String>>()
        .join("");
    let bytes = base64_decode(&base64).unwrap();

    let key_size = (2..=40).fold((0, 0), |size, k| {
        let distance = hamming_distance(&bytes[0..k], &bytes[k..2 * k]) / k as u32;

        if size.1 == 0 || distance < size.1 {
            (k, distance)
        } else {
            size
        }
    }).0;

    let mut blocks: Vec<Vec<u8>> = Vec::new();
    let mut key = String::new();
    for i in 0..key_size {
        blocks.push(bytes.chunks(key_size)
            .filter_map(|c| c.get(i))
            .map(|b| b.clone())
            .collect::<Vec<u8>>())
    }
}

fn fixed_xor(input: &[u8], mask: &[u8]) -> Vec<u8>
{
    return input.into_iter()
        .zip(mask.into_iter())
        .map(|(x, y)| x ^ y)
        .collect::<Vec<u8>>();
}

fn repeating_xor(input: &Vec<u8>, key: &Vec<u8>) -> Vec<u8>
{
    return input.into_iter()
        .zip(key.iter().cycle().into_iter())
        .map(|(x, y)| x ^ y)
        .collect::<Vec<u8>>();
}

fn crack_single_byte_xor(input: &Vec<u8>) -> (u8, Vec<u8>)
{
    let characters = (0..=255).collect::<Vec<u8>>();
    let mut strings: Vec<(u8, Vec<u8>)> = characters.iter()
        .map(|character| {
            let mask = vec![character.clone(); input.len()];
            return (character.clone(), fixed_xor(input, &mask));
        })
        .collect();

    strings.sort_unstable_by(|left, right| sort_by_score_desc(&left.1, &right.1));

    return strings.first().unwrap().clone();
}

fn sort_by_score_desc(left: &Vec<u8>, right: &Vec<u8>) -> Ordering
{
    let left_score = english_score(left);
    let right_score = english_score(right);

    return if left_score > right_score {
        Ordering::Less
    } else if left_score < right_score {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

fn english_score(input: &Vec<u8>) -> i64
{
    let mut score = 0;

    for byte in input {
        if (*byte < 32 || *byte > 122) && *byte != 10 {
            score -= 1000;
        }

        if (*byte >=65 && *byte <=90) || (*byte >= 97 && *byte <= 122) {
            score += 1;
        }
    }

    return score;
}

fn hamming_distance(left: &[u8], right: &[u8]) -> u32
{
    let mut left_bits = BitVec::from_bytes(left);
    let mut right_bits = BitVec::from_bytes(right);
    let left_len = left_bits.len();
    let right_len = right_bits.len();

    if left_len > right_len {
        right_bits.grow(left_len - right_len, false);
    } else if right_len > left_len {
        left_bits.grow(right_len - left_len, false);
    }

    return left_bits.iter()
        .zip(right_bits.iter())
        .fold(0, |sum, (l, r)| sum + if l == r { 0 } else { 1 });
}

#[cfg(test)]
mod test
{
    use std::fs::File;
    use std::io::BufReader;
    use std::io::BufRead;
    use super::*;

    #[test]
    fn case1test()
    {
        let input = "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d";
        let expected = "SSdtIGtpbGxpbmcgeW91ciBicmFpbiBsaWtlIGEgcG9pc29ub3VzIG11c2hyb29t";

        assert_eq!(expected, base64_encode(&decode(input).unwrap()));
    }

    #[test]
    fn case2test()
    {
        let input = "1c0111001f010100061a024b53535009181c";
        let xor = "686974207468652062756c6c277320657965";
        let expected = "746865206b696420646f6e277420706c6179";

        assert_eq!(expected, hex_encode(&fixed_xor(&decode(input).unwrap(), &decode(xor).unwrap())));
    }

    #[test]
    fn case3test()
    {
        let input = "1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736";
        let expected = "Cooking MC's like a pound of bacon";

        assert_eq!(expected, String::from_utf8(crack_single_byte_xor(&decode(input).unwrap()).1).unwrap());
    }

    #[test]
    fn case4test()
    {
        let file = BufReader::new(File::open("4.txt").unwrap());
        let mut lines: Vec<(u8, Vec<u8>)> = file.lines()
            .filter_map(|x| x.ok())
            .map(|x| decode(x).unwrap())
            .map(|x| crack_single_byte_xor(&x))
            .collect();

        lines.sort_unstable_by(|left, right| sort_by_score_desc(&left.1, &right.1));
        assert_eq!("Now that the party is jumping\n", String::from_utf8(lines.first().unwrap().clone().1).unwrap());
    }

    #[test]
    fn case5test()
    {
        let key: Vec<u8> = "ICE".bytes().collect();
        let bytes: Vec<u8> = "Burning 'em, if you ain't quick and nimble
I go crazy when I hear a cymbal".bytes()
            .collect();
        let xord = repeating_xor(&bytes, &key);

        assert_eq!("0b3637272a2b2e63622c2e69692a23693a2a3c6324202d623d63343c2a26226324272765272a282b2f20430a652e2c652a3124333a653e2b2027630c692b20283165286326302e27282f", hex_encode(&xord));
    }

    #[test]
    fn hamming_test()
    {
        let left: Vec<u8> = "this is a test".bytes().collect();
        let right: Vec<u8> = "wokka wokka!!!".bytes().collect();
        assert_eq!(37, hamming_distance(&left, &right));
    }
}

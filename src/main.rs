extern crate hex;
extern crate base64;

use hex::decode;
use hex::encode as hex_encode;
use base64::encode as base64_encode;

fn main() {
}

fn fixed_xor(input: &Vec<u8>, xor: &Vec<u8>) -> Vec<u8>
{
    return input.into_iter()
        .zip(xor.into_iter())
        .map(|(x, y)| x ^ y)
        .collect::<Vec<u8>>();
}

#[cfg(test)]
mod test
{
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
}

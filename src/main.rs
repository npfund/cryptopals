extern crate hex;
extern crate base64;

use hex::decode;
use base64::encode;

fn main() {
    println!("Hello, world!");
}

fn hex_to_base64(hex: &str) -> String
{
    return encode(&decode(hex).unwrap());
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
        assert_eq!(expected, hex_to_base64(&input));
    }
}

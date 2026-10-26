//! Chiffrement du flux TCP : « XRC4Cipher » de SFrame.exe (init FUN_0062ca20, crypt FUN_0062d080).
//!
//! RC4 avec un KSA **non standard** (l'octet de clé est indexé par `j` et non par `i`, et
//! l'accumulateur part de `S[0]`), suivi d'un drop de 1013 octets ; PRGA standard.
//! Une instance par sens et par connexion ; même clé pour l'auth et le jeu.

/// Clé du client NoLimits (SFrame.exe, 0x00dfe098).
pub const DEFAULT_KEY: &[u8] = b"}h79q~B%al;k'y $E";
const DROP: usize = 1013;

#[derive(Clone)]
pub struct XRc4 {
    s: [u8; 256],
    x: u8,
    y: u8,
}

impl XRc4 {
    pub fn new(key: &[u8]) -> Self {
        let mut s: [u8; 256] = core::array::from_fn(|i| i as u8);
        let k: [u8; 256] = core::array::from_fn(|i| key[i % key.len()]);
        let mut j = 0usize;
        let mut t = s[0];
        for i in 0..256 {
            j = (t as usize + k[j] as usize + j) & 0xFF;
            t = s[i];
            s[i] = s[j];
            s[j] = t;
        }
        let mut c = Self { s, x: 0, y: 0 };
        let mut sink = [0u8; DROP];
        c.apply(&mut sink);
        c
    }

    pub fn apply(&mut self, data: &mut [u8]) {
        for b in data {
            self.x = self.x.wrapping_add(1);
            let a = self.s[self.x as usize];
            self.y = self.y.wrapping_add(a);
            self.s[self.x as usize] = self.s[self.y as usize];
            self.s[self.y as usize] = a;
            let idx = self.s[self.x as usize].wrapping_add(self.s[self.y as usize]);
            *b ^= self.s[idx as usize];
        }
    }
}

impl Default for XRc4 {
    fn default() -> Self {
        Self::new(DEFAULT_KEY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }

    #[test]
    fn keystream_vector() {
        // Vecteur de docs/re/network_protocol.md §4.1.
        let mut ks = vec![0u8; 32];
        XRc4::default().apply(&mut ks);
        assert_eq!(ks, hex("5d0c41012b250cfbab96c97300634a583548d5177d7a4d48e7219f62dca00380"));
    }

    #[test]
    fn version_packet_vector() {
        let mut p = hex("1b0000001127533230313530373038300000000000000000000000");
        XRc4::default().apply(&mut p);
        assert_eq!(p, hex("460c41013a025fc99ba7fc43375372683548d5177d7a4d48e7219f"));
    }
}

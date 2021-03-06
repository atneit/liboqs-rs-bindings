use crate as oqs;
use log::trace;
use num::{Integer, NumCast};
use oqs::{Kem, KemBuf, KemWithRejectionSampling, Result, SHA512Buf};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fmt::Display;

pub struct HqcParams<T> {
    pub PARAM_N: T,
    pub PARAM_N1: T,
    pub PARAM_N2: T,
    pub PARAM_N1N2: T,
    pub PARAM_SECURITY: T,
    pub PARAM_DELTA: T,
    pub PARAM_OMEGA: T,
}

pub trait Hqc: KemWithRejectionSampling {
    type X: KemBuf<T = u64>;
    type Y: KemBuf<T = u32>;
    type U: KemBuf<T = u64>;
    type V: KemBuf<T = u64>;
    type Ep: KemBuf<T = u8>;
    type Intermediate: KemBuf<T = u8>;
    type DecoderInput: KemBuf<T = u8>;

    fn params<T: Integer + NumCast>() -> HqcParams<T>;

    fn secrets_from_key(sk: &Self::SecretKey) -> oqs::Result<(Self::X, Self::Y)>;

    fn decode(ct: &mut Self::Ciphertext, sk: &mut Self::SecretKey) -> oqs::Result<Self::Plaintext>;

    #[allow(clippy::type_complexity)]
    fn decode_intermediates(
        ct: &mut Self::Ciphertext,
        sk: &mut Self::SecretKey,
    ) -> oqs::Result<(
        Self::Plaintext,
        Self::Intermediate,
        Self::Intermediate,
        Self::DecoderInput,
        Self::U,
        Self::V,
    )>;

    fn eprime(ct: &mut Self::Ciphertext, sk: &mut Self::SecretKey) -> oqs::Result<Self::Ep>;

    fn eprime_m(
        ct: &mut Self::Ciphertext,
        sk: &mut Self::SecretKey,
        m: &mut Self::Plaintext,
    ) -> oqs::Result<Self::Ep>;

    fn error_components_r1_r2_e(
        pt: &mut Self::Plaintext,
    ) -> oqs::Result<(Self::U, Self::U, Self::U)>;

    fn encaps_with_plaintext_and_r1(
        ct: &mut Self::Ciphertext,
        ss: &mut Self::SharedSecret,
        pk: &mut Self::PublicKey,
        pt: &mut Self::Plaintext,
        r1_sparse: &mut [u32],
    ) -> self::Result<()>;
}

macro_rules! bind_hqc {
    ($($name:ident : {
        PARAM_N: $PARAM_N:expr,
        PARAM_N1: $PARAM_N1:expr,
        PARAM_N2: $PARAM_N2:expr,
        PARAM_N1N2: $PARAM_N1N2:expr,
        PARAM_SECURITY: $PARAM_SECURITY:expr,
        PARAM_DELTA: $PARAM_DELTA:expr,
        PARAM_OMEGA: $PARAM_OMEGA:expr,
        PublicKey: $PK:ident[$PKlen:expr],
        SecretKey : $SK:ident[$SKlen:expr],
        Ciphertext : $CT:ident[$CTlen:expr],
        SharedSecret : $SS:ident[$SSlen:expr],
        Plaintext: $PT:ident[$PTlen:expr],
        X: $X:ident,
        Y: $Y:ident,
        U: $U:ident,
        V: $V:ident,
        Ep: $Ep:ident,
        Intermediate: $Intermediate:ident,
        DecoderInput: $DecoderInput:ident,
        keypair: $keypair:ident,
        secret_key_from_string: $secret_key_from_string:ident,
        encaps: $encaps:ident,
        encaps_with_plaintext: $encaps_with_plaintext:ident,
        numrejections: $numrejections:ident,
        decaps: $decaps:ident,
        parse_ciphertext: $parse_ciphertext:ident,
        decrypt: $decrypt:ident,
        decrypt_intermediates: $decrypt_intermediates:ident,
        eprime: $eprime:ident,
        error_components: $error_components:ident,
    }),+) => {$(

        bind_kem!(
            $name: {
                PublicKey: $PK[$PKlen],
                SecretKey: $SK[$SKlen],
                Ciphertext: $CT[$CTlen],
                SharedSecret: $SS[$SSlen],
                keypair: $keypair,
                encaps: $encaps,
                decaps: $decaps,
            }
        );

        pub struct $PT([u8; $PTlen as usize]);
        impl_kembuf!($PT; u8;$PTlen);

        pub struct $X([u64; ($PARAM_N as usize+63)/64]);
        impl_kembuf!($X; u64;($PARAM_N as usize+63)/64);

        pub struct $Y([u32;  $PARAM_OMEGA]);
        impl_kembuf!($Y; u32; $PARAM_OMEGA);

        pub struct $U([u64; ($PARAM_N as usize+63)/64]);
        impl_kembuf!($U; u64;($PARAM_N as usize+63)/64);

        pub struct $V([u64; ($PARAM_N1N2 as usize+63)/64]);
        impl_kembuf!($V; u64;($PARAM_N1N2 as usize+63)/64);

        pub struct $Ep([u8; ($PARAM_N1N2 as usize+7)/8]);
        impl_kembuf!($Ep; u8;($PARAM_N1N2 as usize+7)/8);

        pub struct $Intermediate([u8; $PARAM_N1 as usize]);
        impl_kembuf!($Intermediate; u8; $PARAM_N1 as usize);

        pub struct $DecoderInput([u8; (($PARAM_N as usize+63)/64)*8]);
        impl_kembuf!($DecoderInput; u8; (($PARAM_N as usize+63)/64)*8);

        impl KemWithRejectionSampling for $name {
            type Plaintext = $PT;

            fn num_rejections(pt: &mut Self::Plaintext) -> Result<u64>
            {
                let pt = pt.as_mut_ptr();
                let mut bytes = 0u64;
                let bptr = (&mut bytes) as *mut u64;
                oqs::calloqs!($numrejections(pt, bptr))?;
                Ok(bytes)
            }

            fn encaps_with_plaintext(
                ct: &mut Self::Ciphertext,
                ss: &mut Self::SharedSecret,
                pk: &mut Self::PublicKey,
                pt: &mut Self::Plaintext,
            ) -> self::Result<()>
            {
                let ct = ct.as_mut_ptr();
                let ss = ss.as_mut_ptr();
                let pk = pk.as_mut_ptr();
                let pt = pt.as_mut_ptr();
                let null = std::ptr::null::<u32>() as *mut u32;
                oqs::calloqs!($encaps_with_plaintext(ct, ss, pk, pt, null, 0))
            }
        }

        impl Hqc for $name {
            type X = $X;
            type Y = $Y;
            type V = $V;
            type U = $U;
            type Ep = $Ep;
            type Intermediate = $Intermediate;
            type DecoderInput = $DecoderInput;

            fn params<T: Integer+NumCast>() -> HqcParams<T> {
                HqcParams {
                    PARAM_N: NumCast::from($PARAM_N).unwrap(),
                    PARAM_N1: NumCast::from($PARAM_N1).unwrap(),
                    PARAM_N2: NumCast::from($PARAM_N2).unwrap(),
                    PARAM_N1N2: NumCast::from($PARAM_N1N2).unwrap(),
                    PARAM_SECURITY: NumCast::from($PARAM_SECURITY).unwrap(),
                    PARAM_DELTA: NumCast::from($PARAM_DELTA).unwrap(),
                    PARAM_OMEGA: NumCast::from($PARAM_OMEGA).unwrap(),
                }
            }

            fn secrets_from_key(sk: &Self::SecretKey) -> oqs::Result<(Self::X, Self::Y)>
            {
                let mut x = Self::X::new();
                let mut y = Self::Y::new();
                let mut pk = Self::PublicKey::new();

                {
                    let x = x.as_mut_ptr();
                    let y = y.as_mut_ptr();
                    let pk = pk.as_mut_ptr();
                    let sk = sk.as_ptr();
                    oqs::calloqs!($secret_key_from_string(x, y, pk, sk))?;
                }

                Ok((x, y))
            }

            fn decode(
                ct: &mut Self::Ciphertext,
                sk: &mut Self::SecretKey,
            ) -> oqs::Result<Self::Plaintext> {
                let mut m = Self::Plaintext::new();
                let mut u = Self::U::new();
                let mut v = Self::V::new();
                let mut d = SHA512Buf::new();
                {
                    let ct = ct.as_mut_ptr();
                    let sk = sk.as_mut_ptr();
                    let u = u.as_mut_ptr();
                    let v = v.as_mut_ptr();
                    let d = d.as_mut_ptr();
                    let m = m.as_mut_ptr();

                    // Parse the ciphertext into u, v and d
                    oqs::calloqs!($parse_ciphertext(u, v, d, ct))?;

                    // Decrypt the ciphertext into m
                    oqs::calloqs!($decrypt(m, u, v, sk))?;
                }
                Ok(m)
            }

            fn decode_intermediates(
                ct: &mut Self::Ciphertext,
                sk: &mut Self::SecretKey,
            ) -> oqs::Result<(Self::Plaintext, Self::Intermediate, Self::Intermediate, Self::DecoderInput, Self::U, Self::V)> {
                let mut m = Self::Plaintext::new();
                let mut u = Self::U::new();
                let mut v = Self::V::new();
                let mut d = SHA512Buf::new();
                let mut enc = Self::Intermediate::new();
                let mut dec = Self::Intermediate::new();
                let mut inp = Self::DecoderInput::new();
                {
                    let ct = ct.as_mut_ptr();
                    let sk = sk.as_mut_ptr();
                    let u = u.as_mut_ptr();
                    let v = v.as_mut_ptr();
                    let d = d.as_mut_ptr();
                    let m = m.as_mut_ptr();
                    let enc = enc.as_mut_ptr();
                    let dec = dec.as_mut_ptr();
                    let inp = inp.as_mut_ptr();

                    // Parse the ciphertext into u, v and d
                    oqs::calloqs!($parse_ciphertext(u, v, d, ct))?;

                    // Decrypt the ciphertext into m
                    oqs::calloqs!($decrypt_intermediates(m, enc, dec, inp, u, v, sk))?;
                }
                Ok((m, enc, dec, inp, u, v))
            }

            fn eprime(ct: &mut Self::Ciphertext, sk: &mut Self::SecretKey) -> oqs::Result<Self::Ep> {
                let mut e = Self::Ep::new();
                let mut m = Self::decode(ct, sk)?;
                let mut u = Self::U::new();
                let mut v = Self::V::new();
                let mut d = SHA512Buf::new();
                {
                    let ct = ct.as_mut_ptr();
                    let sk = sk.as_mut_ptr();
                    let u = u.as_mut_ptr();
                    let v = v.as_mut_ptr();
                    let d = d.as_mut_ptr();
                    let m = m.as_mut_ptr();
                    let e = e.as_mut_ptr();

                    // Parse the ciphertext into u, v and d
                    oqs::calloqs!($parse_ciphertext(u, v, d, ct))?;

                    // Extract eprime
                    oqs::calloqs!($eprime(e, m, u, v, sk))?;
                }
                Ok(e)
            }

            fn eprime_m(ct: &mut Self::Ciphertext, sk: &mut Self::SecretKey, m: &mut Self::Plaintext) -> oqs::Result<Self::Ep> {
                let mut e = Self::Ep::new();
                let mut u = Self::U::new();
                let mut v = Self::V::new();
                let mut d = SHA512Buf::new();
                {
                    let ct = ct.as_mut_ptr();
                    let sk = sk.as_mut_ptr();
                    let u = u.as_mut_ptr();
                    let v = v.as_mut_ptr();
                    let d = d.as_mut_ptr();
                    let m = m.as_mut_ptr();
                    let e = e.as_mut_ptr();

                    // Parse the ciphertext into u, v and d
                    oqs::calloqs!($parse_ciphertext(u, v, d, ct))?;

                    // Extract eprime
                    oqs::calloqs!($eprime(e, m, u, v, sk))?;
                }
                Ok(e)
            }

            fn error_components_r1_r2_e(pt: &mut Self::Plaintext) -> oqs::Result<(Self::U, Self::U, Self::U)>
            {
                let mut r1 = Self::U::new();
                let mut r2 = Self::U::new();
                let mut e = Self::U::new();
                {
                    let m = pt.as_mut_ptr();
                    let r1 = r1.as_mut_ptr();
                    let r2 = r2.as_mut_ptr();
                    let e = e.as_mut_ptr();

                    // Parse the ciphertext into u, v and d
                    oqs::calloqs!($error_components(m, r1, r2, e))?;
                }
                Ok((r1, r2, e))
            }

            fn encaps_with_plaintext_and_r1(
                ct: &mut Self::Ciphertext,
                ss: &mut Self::SharedSecret,
                pk: &mut Self::PublicKey,
                pt: &mut Self::Plaintext,
                r1_sparse: &mut [u32],
            ) -> self::Result<()>
            {
                let ct = ct.as_mut_ptr();
                let ss = ss.as_mut_ptr();
                let pk = pk.as_mut_ptr();
                let pt = pt.as_mut_ptr();
                let r1_sparse_len = r1_sparse.len() as u32;
                let r1_sparse = r1_sparse.as_mut_ptr();
                oqs::calloqs!($encaps_with_plaintext(ct, ss, pk, pt, r1_sparse, r1_sparse_len))
            }
        }
    )*}
}

bind_hqc! (
    Hqc128: {
        PARAM_N: 17669,
        PARAM_N1: 46,
        PARAM_N2: 384,
        PARAM_N1N2: 17664,
        PARAM_SECURITY: 128,
        PARAM_DELTA: 15,
        PARAM_OMEGA: 66,
        PublicKey: Hqc128PublicKey[oqs::OQS_KEM_hqc_128_length_public_key as usize],
        SecretKey : Hqc128SecretKey[oqs::OQS_KEM_hqc_128_length_secret_key as usize],
        Ciphertext : Hqc128Ciphertext[oqs::OQS_KEM_hqc_128_length_ciphertext as usize],
        SharedSecret : Hqc128SharedSecret[oqs::OQS_KEM_hqc_128_length_shared_secret as usize],
        Plaintext : Hqc128Plaintext[oqs::OQS_KEM_hqc_128_length_plaintext as usize],
        X : Hqc128X,
        Y : Hqc128Y,
        U : Hqc128U,
        V : Hqc128V,
        Ep : Hqc128Ep,
        Intermediate : Hqc128Intermediate,
        DecoderInput : Hqc128DecoderInput,
        keypair: OQS_KEM_hqc_128_keypair,
        secret_key_from_string: OQS_KEM_hqc_128_secret_key_from_string,
        encaps: OQS_KEM_hqc_128_encaps,
        encaps_with_plaintext: OQS_KEM_hqc_128_encaps_with_m,
        numrejections: OQS_KEM_hqc_128_numrejections,
        decaps: OQS_KEM_hqc_128_decaps,
        parse_ciphertext: PQCLEAN_HQCRMRS128_CLEAN_hqc_ciphertext_from_string,
        decrypt: PQCLEAN_HQCRMRS128_CLEAN_hqc_pke_decrypt,
        decrypt_intermediates: PQCLEAN_HQCRMRS128_CLEAN_hqc_pke_decrypt_intermediates,
        eprime: PQCLEAN_HQCRMRS128_CLEAN_hqc_pke_eprime,
        error_components: PQCLEAN_HQCRMRS128_CLEAN_hqc_pke_error_components,
    },
    Hqc192: {
        PARAM_N: 35851,
        PARAM_N1: 56,
        PARAM_N2: 640,
        PARAM_N1N2: 35840,
        PARAM_SECURITY: 192,
        PARAM_DELTA: 16,
        PARAM_OMEGA: 100,
        PublicKey: Hqc192PublicKey[oqs::OQS_KEM_hqc_192_length_public_key as usize],
        SecretKey : Hqc192SecretKey[oqs::OQS_KEM_hqc_192_length_secret_key as usize],
        Ciphertext : Hqc192Ciphertext[oqs::OQS_KEM_hqc_192_length_ciphertext as usize],
        SharedSecret : Hqc192SharedSecret[oqs::OQS_KEM_hqc_192_length_shared_secret as usize],
        Plaintext : Hqc192Plaintext[oqs::OQS_KEM_hqc_192_length_plaintext as usize],
        X : Hqc192X,
        Y : Hqc192Y,
        U : Hqc192U,
        V : Hqc192V,
        Ep : Hqc192Ep,
        Intermediate : Hqc192Intermediate,
        DecoderInput : Hqc192DecoderInput,
        keypair: OQS_KEM_hqc_192_keypair,
        secret_key_from_string: OQS_KEM_hqc_192_secret_key_from_string,
        encaps: OQS_KEM_hqc_192_encaps,
        encaps_with_plaintext: OQS_KEM_hqc_192_encaps_with_m,
        numrejections: OQS_KEM_hqc_192_numrejections,
        decaps: OQS_KEM_hqc_192_decaps,
        parse_ciphertext: PQCLEAN_HQCRMRS192_CLEAN_hqc_ciphertext_from_string,
        decrypt: PQCLEAN_HQCRMRS192_CLEAN_hqc_pke_decrypt,
        decrypt_intermediates: PQCLEAN_HQCRMRS192_CLEAN_hqc_pke_decrypt_intermediates,
        eprime: PQCLEAN_HQCRMRS192_CLEAN_hqc_pke_eprime,
        error_components: PQCLEAN_HQCRMRS192_CLEAN_hqc_pke_error_components,
    },
    Hqc256: {
        PARAM_N: 57637,
        PARAM_N1: 90,
        PARAM_N2: 640,
        PARAM_N1N2: 57600,
        PARAM_SECURITY: 256,
        PARAM_DELTA: 29,
        PARAM_OMEGA: 131,
        PublicKey: Hqc256PublicKey[oqs::OQS_KEM_hqc_256_length_public_key as usize],
        SecretKey : Hqc256SecretKey[oqs::OQS_KEM_hqc_256_length_secret_key as usize],
        Ciphertext : Hqc256Ciphertext[oqs::OQS_KEM_hqc_256_length_ciphertext as usize],
        SharedSecret : Hqc256SharedSecret[oqs::OQS_KEM_hqc_256_length_shared_secret as usize],
        Plaintext : Hqc256Plaintext[oqs::OQS_KEM_hqc_256_length_plaintext as usize],
        X : Hqc256X,
        Y : Hqc256Y,
        U : Hqc256U,
        V : Hqc256V,
        Ep : Hqc256Ep,
        Intermediate : Hqc256Intermediate,
        DecoderInput : Hqc256DecoderInput,
        keypair: OQS_KEM_hqc_256_keypair,
        secret_key_from_string: OQS_KEM_hqc_256_secret_key_from_string,
        encaps: OQS_KEM_hqc_256_encaps,
        encaps_with_plaintext: OQS_KEM_hqc_256_encaps_with_m,
        numrejections: OQS_KEM_hqc_256_numrejections,
        decaps: OQS_KEM_hqc_256_decaps,
        parse_ciphertext: PQCLEAN_HQCRMRS256_CLEAN_hqc_ciphertext_from_string,
        decrypt: PQCLEAN_HQCRMRS256_CLEAN_hqc_pke_decrypt,
        decrypt_intermediates: PQCLEAN_HQCRMRS256_CLEAN_hqc_pke_decrypt_intermediates,
        eprime: PQCLEAN_HQCRMRS256_CLEAN_hqc_pke_eprime,
        error_components: PQCLEAN_HQCRMRS256_CLEAN_hqc_pke_error_components,
    }
);

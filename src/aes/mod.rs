// SPDX-FileCopyrightText: Copyright 2023 Savi
// SPDX-License-Identifier: GPL-3.0-only 

use aead::{AeadCore, generic_array::GenericArray};
use aes_gcm_siv::{
    aead::{Aead, KeyInit, OsRng},
    Aes256GcmSiv, Nonce // Or `Aes128GcmSiv`
};
use base64::{Engine as _, engine::general_purpose};
use general_purpose::STANDARD_NO_PAD as BASE64;

#[derive(Clone)]
pub struct AES{
    key: String
}

impl AES{
    /// This function will create a new AES cipher with a random key if no key is provided.
    /// 
    pub fn new(key: Option<String>) -> Self{
        if key.is_some(){
            return AES{
                key: key.unwrap()
            };
        }
        let key = Aes256GcmSiv::generate_key(&mut OsRng);
        let key_base64 = BASE64.encode(key);
        AES{
            key: key_base64
        }
    }

    pub fn get_key(&self) -> &String{
        return &self.key;
    }

    pub fn decrypt(&self, b64_cipher: String) -> String{
        let key_decode = &BASE64.decode(&self.key).unwrap();
        let key = GenericArray::from_slice(key_decode as &[u8]);
    
        let b64_decode = BASE64.decode(b64_cipher).unwrap();
        let nonce = &b64_decode[0..12];
        let ciphertext = &b64_decode[12..b64_decode.len()];
    
        let cipher = Aes256GcmSiv::new(key);
        let decrypted = cipher.decrypt(&Nonce::from_slice(nonce), ciphertext.as_ref()).unwrap();
    
        String::from_utf8(decrypted).unwrap()
    }
    
    pub fn encrypt(&self, message: String) -> String{
        let key_decode = &BASE64.decode(&self.key).unwrap();
        let key = GenericArray::from_slice(key_decode as &[u8]);
    
        let nonce = &Aes256GcmSiv::generate_nonce(&mut OsRng);
        let cipher = Aes256GcmSiv::new(key);
        let ciphertext = cipher.encrypt(nonce, message.as_bytes().as_ref()).unwrap();
    
        let mut nonceciphertext = nonce.to_vec();
        nonceciphertext.extend_from_slice(&ciphertext);
    
        BASE64.encode(nonceciphertext)
    }

}
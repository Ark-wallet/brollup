#[cfg(test)]
mod vse_tests {
    use brollup::{
        noist::setup::{keymap::VSEKeyMap, setup::VSESetup},
        schnorr::Authenticable,
    };

    #[test]
    fn vse_keymap() -> Result<(), String> {
        let signer_1_secret: [u8; 32] =
            hex::decode("396e7f3b89843e1e5610b1fdbaabf1b6a53066f43b22c529f839d69b6799ce8f")
                .unwrap()
                .try_into()
                .unwrap();
        let signer_1_public: [u8; 32] =
            hex::decode("eae0001e445c4f748f91010c1fb6d5b99391e588e605dbbb6ca4e5d98e520cd7")
                .unwrap()
                .try_into()
                .unwrap();

        let signer_2_secret: [u8; 32] =
            hex::decode("31dfea206f96e7b254e00fddb22baac233feb57d6ea98f3fe6929becad1eee78")
                .unwrap()
                .try_into()
                .unwrap();
        let signer_2_public: [u8; 32] =
            hex::decode("25451c1c2d326a14e86c7921cb1467512c944801c4fc0f81f8bd89e85d3ab1f1")
                .unwrap()
                .try_into()
                .unwrap();

        let signer_3_secret: [u8; 32] =
            hex::decode("38e2361ab771574909a9768670fa33406a311a2cae7d446359f09df18ac2cb83")
                .unwrap()
                .try_into()
                .unwrap();
        let signer_3_public: [u8; 32] =
            hex::decode("e8e5393d1873b616c12c6e2bee0c637f58dc5762dda654903c4dd1a72d762c34")
                .unwrap()
                .try_into()
                .unwrap();

        let full_list = vec![signer_1_public, signer_2_public, signer_3_public];

        // Signer 1 keymap.
        let signer_1_keymap =
            VSEKeyMap::new(signer_1_secret, &vec![signer_2_public, signer_3_public]).unwrap();

        if !signer_1_keymap.is_complete(&full_list) {
            return Err(format!("signer_1_keymap is not complete."));
        }

        let signer_1_auth_keymap = match Authenticable::new(signer_1_keymap, signer_1_secret) {
            Some(keymap) => keymap,
            None => return Err(format!("signer_1_auth_keymap err.")),
        };

        if !signer_1_auth_keymap.authenticate() {
            println!("signer_1_auth_keymap auth err.");
        }

        // Signer 2 keymap.
        let signer_2_keymap =
            VSEKeyMap::new(signer_2_secret, &vec![signer_1_public, signer_3_public]).unwrap();

        if !signer_2_keymap.is_complete(&full_list) {
            return Err(format!("signer_2_keymap is not complete."));
        }

        let signer_2_auth_keymap = match Authenticable::new(signer_2_keymap, signer_2_secret) {
            Some(keymap) => keymap,
            None => return Err(format!("signer_2_auth_keymap err.")),
        };

        if !signer_2_auth_keymap.authenticate() {
            println!("signer_2_auth_keymap auth err.");
        }

        // Signer 3 keymap.
        let signer_3_keymap =
            VSEKeyMap::new(signer_3_secret, &vec![signer_1_public, signer_2_public]).unwrap();

        if !signer_3_keymap.is_complete(&full_list) {
            return Err(format!("signer_3_keymap is not complete."));
        }

        let signer_3_auth_keymap = match Authenticable::new(signer_3_keymap, signer_3_secret) {
            Some(keymap) => keymap,
            None => return Err(format!("signer_3_auth_keymap err.")),
        };

        if !signer_3_auth_keymap.authenticate() {
            println!("signer_3_auth_keymap auth err.");
        }

        let mut vse_setup = VSESetup::new(&full_list, 0);

        if !vse_setup.insert(signer_1_auth_keymap) {
            return Err(format!("signer_1_auth_keymap insert err."));
        };

        if !vse_setup.insert(signer_2_auth_keymap) {
            return Err(format!("signer_2_auth_keymap insert err."));
        };

        if !vse_setup.insert(signer_3_auth_keymap) {
            return Err(format!("signer_3_auth_keymap insert err."));
        };

        if !vse_setup.validate() {
            return Err(format!("vse_setup validate err."));
        }

        Ok(())
    }
}

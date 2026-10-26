use crate::errors::{Result, crabwiseError};

/// Represents the connection state of a Stellar wallet.
#[derive(Clone, Debug, PartialEq)]
pub enum WalletConnectionState {
    Disconnected,
    Connecting,
    Connected { public_key: String },
    Error { message: String },
}

/// Minimal wallet connection abstraction.
pub struct Wallet {
    state: WalletConnectionState,
}

impl Wallet {
    pub fn new() -> Self {
        Self { state: WalletConnectionState::Disconnected }
    }

    pub fn state(&self) -> &WalletConnectionState {
        &self.state
    }

    /// Attempt to connect to a Stellar wallet (browser WASM only).
    pub fn connect(&mut self) -> Result<&str> {
        #[cfg(target_arch = "wasm32")]
        {
            self.state = WalletConnectionState::Connecting;
            match self.try_wasm_connect() {
                Ok(pk) => {
                    self.state = WalletConnectionState::Connected { public_key: pk.clone() };
                    Ok(Box::leak(pk.into_boxed_str()))
                }
                Err(e) => {
                    self.state = WalletConnectionState::Error { message: e.to_string() };
                    Err(e)
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state = WalletConnectionState::Error {
                message: "wallet connection is only available in a browser (wasm32) environment".into(),
            };
            Err(crabwiseError::WalletConnection("not a wasm target".into()))
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn try_wasm_connect(&self) -> Result<String> {
        Err(crabwiseError::WalletConnection("Freighter / Albedo not detected".into()))
    }

    pub fn disconnect(&mut self) {
        self.state = WalletConnectionState::Disconnected;
    }

    pub fn public_key(&self) -> Result<&str> {
        match &self.state {
            WalletConnectionState::Connected { public_key } => Ok(public_key),
            WalletConnectionState::Disconnected => {
                Err(crabwiseError::WalletConnection("wallet is disconnected".into()))
            }
            WalletConnectionState::Connecting => {
                Err(crabwiseError::WalletConnection("wallet connection is in progress".into()))
            }
            WalletConnectionState::Error { message } => {
                Err(crabwiseError::WalletConnection(format!("previous error: {}", message)))
            }
        }
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_starts_disconnected() {
        let w = Wallet::new();
        assert_eq!(w.state(), &WalletConnectionState::Disconnected);
    }

    #[test]
    fn test_connect_returns_error_on_non_wasm() {
        let mut w = Wallet::new();
        let result = w.connect();
        assert!(result.is_err());
        assert!(matches!(w.state(), WalletConnectionState::Error { .. }));
    }

    #[test]
    fn test_disconnect_resets_state() {
        let mut w = Wallet::new();
        let _ = w.connect();
        w.disconnect();
        assert_eq!(w.state(), &WalletConnectionState::Disconnected);
    }

    #[test]
    fn test_public_key_returns_error_when_disconnected() {
        let w = Wallet::new();
        let result = w.public_key();
        assert!(result.is_err());
    }
}

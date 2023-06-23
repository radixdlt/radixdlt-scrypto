use crate::prelude::*;

#[derive(Clone, Copy)]
pub struct TransactionHashDisplayContext<'a> {
    pub encoder: Option<&'a TransactionHashBech32Encoder>,
}

impl<'a> TransactionHashDisplayContext<'a> {
    pub fn with_encoder(encoder: &'a TransactionHashBech32Encoder) -> Self {
        TransactionHashDisplayContext {
            encoder: Some(encoder),
        }
    }
}

impl<'a> From<&'a TransactionHashBech32Encoder> for TransactionHashDisplayContext<'a> {
    fn from(encoder: &'a TransactionHashBech32Encoder) -> Self {
        Self::with_encoder(encoder)
    }
}

impl<'a> From<Option<&'a TransactionHashBech32Encoder>> for TransactionHashDisplayContext<'a> {
    fn from(encoder: Option<&'a TransactionHashBech32Encoder>) -> Self {
        Self { encoder }
    }
}

impl<'a> ContextualDisplay<TransactionHashDisplayContext<'a>> for IntentHash {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &TransactionHashDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            encoder.encode_to_fmt(f, self).map_err(|_| fmt::Error)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl<'a> ContextualDisplay<TransactionHashDisplayContext<'a>> for SignedIntentHash {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &TransactionHashDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            encoder.encode_to_fmt(f, self).map_err(|_| fmt::Error)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl<'a> ContextualDisplay<TransactionHashDisplayContext<'a>> for NotarizedTransactionHash {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &TransactionHashDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        if let Some(encoder) = context.encoder {
            encoder.encode_to_fmt(f, self).map_err(|_| fmt::Error)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

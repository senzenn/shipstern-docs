// #region mint-extensions
use shipstern::{Handler, HandlerResult};
use shipstern_core::AccountUpdate;
use shipstern_spl_token_extensions_parser::{account, NativeExtensionData, TokenExtensionState};
use spl_token_2022::extension::ExtensionType;

#[derive(Debug)]
pub struct MintExtensions;

impl Handler<TokenExtensionState, AccountUpdate> for MintExtensions {
    async fn handle(&self, value: &TokenExtensionState, raw: &AccountUpdate) -> HandlerResult<()> {
        let Some(account::Account::ExtendedMint(mint)) = &value.account else {
            return Ok(());
        };
        if let Some(base) = &mint.base_account {
            tracing::info!(
                slot = raw.slot,
                supply = base.supply,
                decimals = base.decimals
            );
        }
        for extension in &mint.extensions {
            let kind = ExtensionType::try_from(u16::try_from(extension.extension_type)?)?;
            match NativeExtensionData::try_from((kind, extension.data.as_slice())) {
                Ok(decoded) => tracing::info!(?kind, ?decoded, "mint extension"),
                Err(error) => tracing::warn!(?kind, ?error, "extension not decoded"),
            }
        }
        Ok(())
    }
}
// #endregion mint-extensions

// #region register-extensions
pub fn mint_pipeline(
) -> shipstern::Pipeline<shipstern_spl_token_extensions_parser::AccountParser, [MintExtensions; 1]>
{
    shipstern::Pipeline::new(
        shipstern_spl_token_extensions_parser::AccountParser,
        [MintExtensions],
    )
}
// #endregion register-extensions

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_extension_decodes_but_permissioned_burn_is_unsupported() {
        assert!(matches!(
            NativeExtensionData::try_from((ExtensionType::ImmutableOwner, &[][..]))
                .expect("immutable owner marker should decode"),
            NativeExtensionData::ImmutableOwner(_)
        ));
        assert!(NativeExtensionData::try_from((ExtensionType::PermissionedBurn, &[][..])).is_err());
    }
}

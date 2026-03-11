use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_VALIDATE_LOAN: u32 = comp_def_offset("validate_loan");
const COMP_DEF_OFFSET_CHECK_LIQUIDATION: u32 = comp_def_offset("check_liquidation");
const COMP_DEF_OFFSET_BORROW_CAPACITY: u32 = comp_def_offset("compute_borrow_capacity");

declare_id!("YOUR_PROGRAM_ID_HERE");

#[arcium_program]
pub mod private_lending {
    use super::*;

    pub fn init_validate_loan_comp_def(ctx: Context<InitValidateLoanCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    pub fn init_check_liquidation_comp_def(ctx: Context<InitCheckLiquidationCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    pub fn init_borrow_capacity_comp_def(ctx: Context<InitBorrowCapacityCompDef>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    pub fn validate_loan(
        ctx: Context<ValidateLoan>,
        computation_offset: u64,
        ciphertext: [u8; 32],
        pub_key: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        let args = ArgBuilder::new()
            .x25519_pubkey(pub_key)
            .plaintext_u128(nonce)
            .encrypted_u64(ciphertext)
            .build();
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![ValidateLoanCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[],
            )?],
            1,
            0,
        )?;
        emit!(LoanRequestSubmitted {
            requester: ctx.accounts.payer.key(),
            computation_offset,
        });
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "validate_loan")]
    pub fn validate_loan_callback(
        ctx: Context<ValidateLoanCallback>,
        output: SignedComputationOutputs<ValidateLoanOutput>,
    ) -> Result<()> {
        let eligible_enc = match output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        ) {
            Ok(ValidateLoanOutput { field_0 }) => field_0,
            Err(e) => {
                msg!("Error: {}", e);
                return Err(ErrorCode::AbortedComputation.into());
            }
        };
        emit!(LoanEligibilityResult {
            eligible_encrypted: eligible_enc.ciphertexts[0],
            nonce: eligible_enc.nonce.to_le_bytes(),
        });
        Ok(())
    }

    pub fn check_liquidation(
        ctx: Context<CheckLiquidation>,
        computation_offset: u64,
        ciphertext: [u8; 32],
        pub_key: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        let args = ArgBuilder::new()
            .x25519_pubkey(pub_key)
            .plaintext_u128(nonce)
            .encrypted_u64(ciphertext)
            .build();
        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            vec![CheckLiquidationCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[],
            )?],
            1,
            0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "check_liquidation")]
    pub fn check_liquidation_callback(
        ctx: Context<CheckLiquidationCallback>,
        output: SignedComputationOutputs<CheckLiquidationOutput>,
    ) -> Result<()> {
        let result_enc = match output.verify_output(
            &ctx.accounts.cluster_account,
            &ctx.accounts.computation_account,
        ) {
            Ok(CheckLiquidationOutput { field_0 }) => field_0,
            Err(e) => {
                msg!("Error: {}", e);
                return Err(ErrorCode::AbortedComputation.into());
            }
        };
        emit!(LiquidationCheckResult {
            result_encrypted: result_enc.ciphertexts[0],
            nonce: result_enc.nonce.to_le_bytes(),
        });
        Ok(())
    }
}

#[event]
pub struct LoanRequestSubmitted {
    pub requester: Pubkey,
    pub computation_offset: u64,
}

#[event]
pub struct LoanEligibilityResult {
    pub eligible_encrypted: [u8; 32],
    pub nonce: [u8; 16],
}

#[event]
pub struct LiquidationCheckResult {
    pub result_encrypted: [u8; 32],
    pub nonce: [u8; 16],
}

#[error_code]
pub enum ErrorCode {
    #[msg("MPC computation was aborted")]
    AbortedComputation,
}
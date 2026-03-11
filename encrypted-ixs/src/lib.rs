use arcis::*;

#[encrypted]
mod circuits {
    use arcis::*;

    #[derive(Copy, Clone)]
    pub struct LoanRequest {
        pub borrow_amount: u64,
        pub collateral_amount: u64,
    }

    #[derive(Copy, Clone)]
    pub struct LoanPosition {
        pub borrow_amount: u64,
        pub collateral_amount: u64,
        pub interest_bps: u64,
    }

    #[instruction]
    pub fn validate_loan(
        request: Enc<Shared, LoanRequest>,
    ) -> Enc<Shared, bool> {
        let r = request.to_arcis();
        let collateral_scaled = r.collateral_amount * 10000u64;
        let borrow_scaled = r.borrow_amount * 15000u64;
        let eligible = collateral_scaled >= borrow_scaled;
        request.owner.from_arcis(eligible)
    }

    #[instruction]
    pub fn check_liquidation(
        position: Enc<Shared, LoanPosition>,
    ) -> Enc<Shared, bool> {
        let p = position.to_arcis();
        let interest = (p.borrow_amount * p.interest_bps) / 10000u64;
        let total_debt = p.borrow_amount + interest;
        let collateral_ratio_bps = (p.collateral_amount * 10000u64) / total_debt;
        let should_liquidate = collateral_ratio_bps < 12000u64;
        position.owner.from_arcis(should_liquidate)
    }

    #[instruction]
    pub fn compute_borrow_capacity(
        position: Enc<Shared, LoanPosition>,
    ) -> Enc<Shared, u64> {
        let p = position.to_arcis();
        let max_total_borrow = (p.collateral_amount * 10000u64) / 15000u64;
        let capacity = if max_total_borrow > p.borrow_amount {
            max_total_borrow - p.borrow_amount
        } else {
            0u64
        };
        position.owner.from_arcis(capacity)
    }
}

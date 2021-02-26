/*
 *
 */

// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{wee_alloc, AccountId, Balance};
use near_sdk::{env, near_bindgen};
use std::collections::HashMap;
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::serde::{Deserialize, Serialize};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

//static PROJECT_KEY: std::String = String::from("state");

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum TaskStatus {
    BLOCKED,
    ASSIGNED,
    IN_PROGRESS,
    COMPLETED
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Expense {
    label: String,
    amount: Balance
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProjectFunding {
    total: Balance,
    spent: Balance,
    expenses: Vector<Expense>
}

impl ProjectFunding {
    pub fn with_amount(amount: Balance) -> Self {
        let mut proj_funding = ProjectFunding::default();
        proj_funding.total = amount;
        proj_funding
    }
}

impl Default for ProjectFunding {
    fn default() -> Self {
        ProjectFunding { total: Balance::default(), spent: Balance::default(), expenses: Expense::get_empty_expenses_vector() }
    }
}




impl Expense {
    pub fn get_empty_expenses_vector() -> Vector<Expense> {
        Vector::<Expense>::new(b"e".to_vec())
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProjectDetails {
    title: String,
    description: String
}

/*
* TODO: answer: how do contributions affect the overall project budget? Do their amounts
*  get counted towards the 'spent' when status = COMPLETED?
*/

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Contribution {
    account: AccountId,
    task: String,
    amount: Balance,
    status: TaskStatus
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Project {
    factory: AccountId,
    proposal: AccountId,
    details: Option<ProjectDetails>,
    funding: Option<ProjectFunding>,
    contributors: UnorderedMap<AccountId,Contribution>,
}

impl Default for Project {
    fn default() -> Self {
        env::panic(b"The contract is not initialized.")
    }

}
#[near_bindgen]
impl Project {
    #[init]
    pub fn new() -> Self {
        // Useful snippet to copy/paste, making sure state isn't already initialized
        assert!(env::state_read::<Self>().is_none(), "Already initialized");
        // Note this is an implicit "return" here
        Self {
            factory:env::predecessor_account_id(),
            proposal:env::signer_account_id(),
            contributors: UnorderedMap::new(b"c".to_vec()),
            details: None,
            funding: None
        }
    }
    pub fn is_configured(&self) -> bool { Option::is_some(&self.details)}

    pub fn assert_configured(&self) -> () { assert!(self.is_configured(),"Not configured project"); }

    pub fn configure(&mut self, title: String, description: String) -> () {
        self.details = Some(ProjectDetails {title, description});
        self.funding = Some(ProjectFunding::with_amount(env::account_balance()-near_sdk::Balance::MIN))
    }
    pub fn add_funds(&mut self) -> () {
        self.assert_configured();
        if let Some(funding) = &mut self.funding {
           funding.total = funding.total + env::attached_deposit();
        }
    }
    /*
     * TODO: why do we need to include the account param here, if it is already embedded within the contribution object?
    */
    pub fn add_contributor(&mut self,account: AccountId,contribution: Contribution
    ) -> () {
        self.assert_configured();
        &self.contributors.insert(&account,&contribution);
    }

    /**
     * @function add_expense
     * @param label {string} - expense label
     * @param amount  - expense amount
     *
     * Track an expense.
     *
     * TODO: find out if it is better to decompose types into the contract interface like this
     *  to save on serde costs... or better to keep the custom types exposed like in add_contributor()
     *  for better readability?
     */
    pub fn add_expense(&mut self, label: String, amount: Balance) -> () {
        self.assert_configured();

        let expense = Expense { label, amount };
        if let Some(funding) = &mut self.funding {
            funding.expenses.push(&expense);
            funding.spent = funding.spent + amount;
        }
    }
    // pub fn get_project(&self) -> &Project {
    //     return self;
    // }

    pub fn get_factory(self) -> AccountId {
        self.factory
    }
    pub fn get_proposal(self) -> AccountId {
        self.proposal
    }

    pub fn get_remaining_budget(&self) -> Balance {
        self.assert_configured();
        if let Some(funding) = &self.funding {
            funding.total - funding.spent
        } else {
            0
        }
    }

    // pub fn get_expenses(self) -> Option<Vector<Expense>> {
    //     if let Some(funding) = self.funding {
    //         Some(funding.expenses)
    //     } else {
    //         None
    //     }
    // }
    // pub fn get_contributors(self) -> UnorderedMap<AccountId,Contribution> {
    //     self.contributors
    // }
}
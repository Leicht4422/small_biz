#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Inventory {
    id: u64,
    name: String,
    description: String,
    quantity: u32,
    amount: f64,
    created_at: u64,
    updated_at: Option<u64>,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for Inventory {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for Inventory {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Sale {
    id: u64,
    name: String,
    description: Option<String>,
    quantity: u32,
    amount: f64,
    timestamp: u64,
    store_id: u64,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for Sale {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for Sale {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Expense {
    id: u64,
    name: String,
    description: String,
    amount: f64,
    timestamp: u64,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for Expense {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for Expense {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static INV_STORAGE: RefCell<StableBTreeMap<u64, Inventory, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static SALE_STORAGE: RefCell<StableBTreeMap<u64, Sale, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static EXPENSE_STORAGE: RefCell<StableBTreeMap<u64, Expense, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));
}

//Payloads for adding and updating 
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct InventoryPayload {
    name: String,
    description: String,
    quantity: u32,
    amount: f64
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct SalePayload {
    name: String,
    description: Option<String>,
    quantity: u32,
    amount: f64,
    store_id: u64,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct ExpensePayload {
    name: String,
    description: String,
    amount: f64
}

//Retrieve a specific entry in the Inventory
#[ic_cdk::query]
fn get_inventory(id: u64) -> Result<Inventory, Error> {
    match _get_inventory(&id) {
        Some(inventory) => Ok(inventory),
        None => Err(Error::NotFound {
            msg: format!("Item with id={} not found", id),
        }),
    }
}

fn _get_inventory(id: &u64) -> Option<Inventory> {
    INV_STORAGE.with(|service| service.borrow().get(id))
}

//Create a new entry in the Inventory
#[ic_cdk::update]
fn add_inventory(payload: InventoryPayload) -> Option<Inventory> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let inventory = Inventory {
        id,
        name: payload.name,
        description: payload.description,
        quantity: payload.quantity,
        amount: payload.amount,
        created_at: time(),
        updated_at: None,
    };
    do_insert(&inventory);
    Some(inventory)
}

//Update the inventory
#[ic_cdk::update]
fn update_inventory(id: u64, payload: InventoryPayload) -> Result<Inventory, Error> {
    match INV_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut inventory) => {
            inventory.name = payload.name;
            inventory.description =  payload.description;
            inventory.quantity =  payload.quantity;
            inventory.updated_at = Some(time());
            do_insert(&inventory);
            Ok(inventory)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "Item in Inventory with id={}. is not found",id),
        }),
    }
}

fn do_insert(inventory: &Inventory) {
    INV_STORAGE.with(|service| service.borrow_mut().insert(inventory.id, inventory.clone()));
}

//Delete an entry in the Inventory
#[ic_cdk::update]
fn delete_inventory(id: u64) -> Result<Inventory, Error> {
    match INV_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(inventory) => Ok(inventory),
        None => Err(Error::NotFound {
            msg: format!(
                "Item in Inventory with id={}. is not found",id),
        }),
    }
}

//Retrieve a sale
#[ic_cdk::query]
fn get_sale(id: u64) -> Result<Sale, Error> {
    match _get_sale(&id) {
        Some(inventory) => Ok(inventory),
        None => Err(Error::NotFound {
            msg: format!("Sale with id={} not found", id),
        }),
    }
}

fn _get_sale(id: &u64) -> Option<Sale> {
    SALE_STORAGE.with(|service| service.borrow().get(id))
}

//Create a new Sale
#[ic_cdk::update]
fn add_sale(payload: SalePayload) -> Option<Sale> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
        // Use store_id from the sale payload to find the corresponding inventory item
    let inventory_id = payload.store_id;
    match INV_STORAGE.with(|service| service.borrow().get(&inventory_id)) {
        Some(mut inventory_item) => {
            // Check if there is enough quantity in the inventory
            if inventory_item.quantity >= payload.quantity {
                // Decrement the quantity in the inventory
                inventory_item.quantity -= payload.quantity;
                do_insert(&inventory_item); // Update the inventory
            } else {
                return None;
            }
        }
        None => {
            return None;
        }
    }
    let sale = Sale {
        id,
        name: payload.name,
        description: payload.description,
        quantity: payload.quantity,
        amount: payload.amount,
        store_id: payload.store_id,
        timestamp: time(),
    };
    do_insert_sale(&sale);
    Some(sale)
}

//Update a sale
#[ic_cdk::update]
fn update_sale(id: u64, payload: SalePayload) -> Result<Sale, Error> {
    match SALE_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut sale) => {
            sale.name = payload.name;
            sale.description =  payload.description;
            sale.quantity =  payload.quantity;
            sale.store_id = payload.store_id;
            sale.amount = payload.amount;
            sale.timestamp = time();
            do_insert_sale(&sale);
            Ok(sale)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "Sale in Inventory with id={}. is not found",id),
        }),
    }
}

fn do_insert_sale(sale: &Sale) {
    SALE_STORAGE.with(|service| service.borrow_mut().insert(sale.id, sale.clone()));
}

//Delete a Sale
#[ic_cdk::update]
fn delete_sale(id: u64) -> Result<Sale, Error> {
    match SALE_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(sale) => Ok(sale),
        None => Err(Error::NotFound {
            msg: format!(
                "Sale with id={}. is not found",id),
        }),
    }
}

//Retrive a specific expense
#[ic_cdk::query]
fn get_expense(id: u64) -> Result<Expense, Error> {
    match _get_expense(&id) {
        Some(expense) => Ok(expense),
        None => Err(Error::NotFound {
            msg: format!("Expense with id={} not found", id),
        }),
    }
}

fn _get_expense(id: &u64) -> Option<Expense> {
    EXPENSE_STORAGE.with(|service| service.borrow().get(id))
}

//Create a new Expense entry
#[ic_cdk::update]
fn add_expense(payload: ExpensePayload) -> Option<Expense> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");
    let expense = Expense {
        id,
        name: payload.name,
        description: payload.description,
        amount: payload.amount,
        timestamp: time(),
    };
    do_insert_expense(&expense);
    Some(expense)
}

//Update an expense
#[ic_cdk::update]
fn update_expense(id: u64, payload: ExpensePayload
) -> Result<Expense, Error> {
    match EXPENSE_STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut expense) => {
            expense.name = payload.name;
            expense.description =  payload.description;
            expense.amount = payload.amount;
            expense.timestamp = time();
            do_insert_expense(&expense);
            Ok(expense)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "Expense in Inventory with id={}. is not found",id),
        }),
    }
}

fn do_insert_expense(expense: &Expense) {
    EXPENSE_STORAGE.with(|service| service.borrow_mut().insert(expense.id, expense.clone()));
}

//Delete an expense
#[ic_cdk::update]
fn delete_expense(id: u64) -> Result<Expense, Error> {
    match EXPENSE_STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(sale) => Ok(sale),
        None => Err(Error::NotFound {
            msg: format!(
                "Expense with id={}. is not found",id),
        }),
    }
}

//Retrieve all Expenses
#[ic_cdk::query]
fn get_all_expenses() -> Result<Vec<Expense>, Error> {
    let expenses_map: Vec<(u64, Expense)> = EXPENSE_STORAGE.with(|service| service.borrow().iter().collect());
    let expenses: Vec<Expense> = expenses_map.into_iter().map(|(_, expense)| expense).collect();

    if !expenses.is_empty() {
        Ok(expenses)
    } else {
        Err(Error::NotFound {
            msg: "No expenses found.".to_string(),
        })
    }
}

//Retieve all Sales
#[ic_cdk::query]
fn get_all_sales() -> Result<Vec<Sale>, Error> {
    let sale_map: Vec<(u64, Sale)> = SALE_STORAGE.with(|service| service.borrow().iter().collect());
    let sale: Vec<Sale> = sale_map.into_iter().map(|(_, sale)| sale).collect();

    if !sale.is_empty() {
        Ok(sale)
    } else {
        Err(Error::NotFound {
            msg: "No sales found.".to_string(),
        })
    }
}

//Retrieve all items in Inventory
#[ic_cdk::query]
fn get_all_inventory() -> Result<Vec<Inventory>, Error> {
    let inventory_map: Vec<(u64, Inventory)> = INV_STORAGE.with(|service| service.borrow().iter().collect());
    let inventory: Vec<Inventory> = inventory_map.into_iter().map(|(_, inventory)| inventory).collect();

    if !inventory.is_empty() {
        Ok(inventory)
    } else {
        Err(Error::NotFound {
            msg: "Empty Inventory Refill soon .".to_string(),
        })
    }
}

//Total Sales
#[ic_cdk::query]
fn calculate_total_sales_amount() -> Result<f64, Error> {
    let total_amount: f64 = SALE_STORAGE
        .with(|service| {
            service
                .borrow()
                .iter()
                .map(|(_, sale)| sale.amount)
                .sum()
        });

    Ok(total_amount)
}

//Total Expenses
#[ic_cdk::query]
fn calculate_total_expenses_amount() -> Result<f64, Error> {
    let total_expenses: f64 = EXPENSE_STORAGE
        .with(|service| {
            service
                .borrow()
                .iter()
                .map(|(_, expense)| expense.amount)
                .sum()
        });

    Ok(total_expenses)
}

//Total Inventory amount
#[ic_cdk::query]
fn calculate_total_inv_amount() -> Result<f64, Error> {
    let total_inv_amount: f64 = INV_STORAGE
        .with(|service| {
            service
                .borrow()
                .iter()
                .map(|(_, inventory)| inventory.amount)
                .sum()
        });

    Ok(total_inv_amount)
}

//search function to retrieve a specific item in Inventory
#[ic_cdk::query]
fn search_inventory_by_name_wrapper(name: String) -> Result<Vec<Inventory>, Error> {
    search_inventory_by_name(&name)
}

fn search_inventory_by_name<'a>(name: &'a str) -> Result<Vec<Inventory>, Error> {
    let matching_items: Vec<Inventory> = INV_STORAGE
        .with(|service| {
            service
                .borrow()
                .iter()
                .filter(|(_, item)| item.name.contains(name))
                .map(|(_, item)| item.clone())
                .collect()
        });

    if !matching_items.is_empty() {
        Ok(matching_items)
    } else {
        Err(Error::NotFound {
            msg: format!("No inventory items found with name: {}", name),
        })
    }
}





#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

ic_cdk::export_candid!();

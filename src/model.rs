/*
DESCRIPTION OF APP FLOW

CustomerOrder should be created when user gets to page
then when they click a link the item is added to their order <Vec>
we will have text that is a link so example:

Robusta - Strong and bitter

this will have a <a href> and will contain a url with the info for the order

something like this
<a href="/add?coffee=Robusta&size=Small&qty=2">2 Small Robusta</a>

use get request to get the info
GET /add?coffee=Arabica&size=Large&qty=2


then create itemOrder object using info

then add to customer order


then update inventory
*/
use std::collections::HashMap;


#[derive(Debug, Clone, Copy)]
pub enum Size{
    Small,
    Medium,
    Large
}


impl Size{
    //so we can calculate price on the backend inside the struct when we create a new item order
    fn price(&self) -> f32{
        match self {
            Size::Small => 5.0,

            Size::Medium => 8.0,

            Size::Large => 12.0
        }
    }

    //how much stock to take out when reducing stock
    pub fn amount(&self) ->i32{
        match self{
            Size::Small => 1,
            Size::Medium => 2,
            Size::Large => 3
        }
    }

    //to_str method
    pub fn to_str(&self) -> &str {
        match self {
            Size::Small => "Small",
            Size::Medium => "Medium",
            Size::Large => "Large",
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum Roast{
    Light,
    Medium,
    Dark
}


impl Roast{
    //for insertion into database
    pub fn to_str(&self) -> &str {
        match self {
            Roast::Light => "Light",
            Roast::Medium => "Medium",
            Roast::Dark => "Dark",
        }
    }
}

//coffee enum just so we have fixed types, if we were building this to dynamically add coffees
//then string could be better but this will be easier to manage errors


#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Coffee{
    Columbian,
    Arabica,
    Robusta,
    Excelsa,
    BreakfastBlend,
    MidnightRoast
}

#[derive(Debug, Clone)]
pub struct Inventory{
    pub stock : HashMap<Coffee, i32>
}


//basically just gonna do number of bags available
//disregard the large,small,medium that we can implement later
//so with the simulation just gonna remove one bag per purchase

impl Inventory{
    //this will take in the stock values from the database
    pub fn new(stock: HashMap<Coffee,i32>) -> Self{

        Inventory { stock }

    }

    pub fn add_stock(&mut self, coffee : Coffee, amount : i32){
        //or insert checks if a value exists at the location at thekey, returns mutable reference to the value
        let inventory_add = self.stock.entry(coffee).or_insert(0);

        //dereference the pointer here
        *inventory_add += amount
    } 


    //if not enough coffee to remove from return false here
    pub fn reduce_stock(&mut self, coffee : Coffee, amount : i32) -> bool{

        //get_mut return mutable reference for the value at the key location in the hashmap
        if let Some(current_stock) = self.stock.get_mut(&coffee){
            if *current_stock >= amount{
                *current_stock -= amount;
                return true;
            }
            
        }
        false
    }

}

impl Coffee{
    //description: we didn't really ednd up using this
    pub fn description(&self) -> &'static str{
        match self {
            Coffee::Columbian => "Smooth and balanced with mild acidity",
            Coffee::Arabica => "Sweet and complex with fruity notes",
            Coffee::Robusta => "Strong and bitter with high caffeine",
            Coffee::Excelsa => "Tart and fruity with a unique profile",
            Coffee::BreakfastBlend => "Light and bright, perfect for mornings",
            Coffee::MidnightRoast => "Dark and bold with deep flavor",
            
        }
    }
    //simple to_string func
    pub fn to_str(&self) -> &str {
        match self {
            Coffee::Columbian => "Columbian",
            Coffee::Arabica => "Arabica",
            Coffee::Robusta => "Robusta",
            Coffee::Excelsa => "Excelsa",
            Coffee::BreakfastBlend => "BreakfastBlend",
            Coffee::MidnightRoast => "MidnightRoast",
        }
    }
}



//might need lifetimes('a things) for the coffeeitem in the parameters
//this would be represented by an object like 2 bags og Columbian Dark size L which would then be 24 as price
//ItemOrder contains a customer's order of coffee
#[derive(Debug, Clone)]
pub struct ItemOrder{
    pub coffee : Coffee,
    pub roast : Roast,
    pub size : Size,
    pub quantity : i32,
    pub price : f32
}



impl ItemOrder{

    fn new(new_coffee : Coffee, new_roast : Roast, new_size : Size, new_quantity : i32) -> Self{

        let quantity_float = new_quantity as f32;
        //use price method to get total price 
        let total_price = new_size.price() * quantity_float;

        ItemOrder{
            coffee : new_coffee,
            size : new_size,
            quantity : new_quantity,
            price : total_price,
            roast : new_roast
        }
    }

}


//customerOrder struct will basically be like a receipt of everything they bought
#[derive(Debug, Clone)]
pub struct CustomerOrder{
    pub id : i32,
    pub items : Vec<ItemOrder>,
    pub total_price : f32
}   


impl CustomerOrder{
    pub fn new(new_id : i32 ) -> Self{

        CustomerOrder{
            id : new_id,
            items : Vec::new(),
            total_price : 0.0
        }
    }
    //add item to struct
    pub fn add_item(&mut self, coffee: Coffee, roast: Roast, size: Size, quantity: i32){

        let item = ItemOrder::new(coffee, roast, size, quantity);
        self.total_price += item.price;
        self.items.push(item);

    }
}


//unit tests ran with cargo test
#[cfg(test)]
mod tests{

    use super::*;

    #[test] //test adding stock
    fn add_stock(){
        let mut hash = HashMap::new();
        hash.insert(Coffee::Arabica,100);
        let mut inventory = Inventory::new(hash);
        inventory.add_stock(Coffee::Arabica, 20);
        assert_eq!(inventory.stock[&Coffee::Arabica], 120)
    }

    #[test] //test reducing stock
    fn reduce_stock_insufficient(){
        let mut hash = HashMap::new();
        hash.insert(Coffee::Columbian,100);
        let mut inventory = Inventory::new(hash);

        //reduce_stock returns a bool(true if succesful false if not enough stock in inventory)
        let result = inventory.reduce_stock(Coffee::Columbian, 101);
        assert!(!result);
        assert_eq!(inventory.stock[&Coffee::Columbian], 100);
    }


    #[test] //test item pricing if we add items
    fn test_add_items_total(){
        let mut order = CustomerOrder::new(1);

        order.add_item(Coffee::Arabica, Roast::Medium, Size::Large, 2);

        assert_eq!(order.total_price, 24.0);
    }

    #[test] //test creating a new order
    fn test_new_order(){
        let order = CustomerOrder::new(2);
        assert_eq!(order.total_price, 0.0);
        assert_eq!(order.items.len(), 0);
    }

}
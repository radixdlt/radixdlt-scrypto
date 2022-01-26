use scrypto::prelude::*;

// ElfWorkshop component. 
// People can register as elf to receive a badge.
// They can then use the badge to create new toys and the component
// keeps track of the amount of toys each elf created.
blueprint! {
    struct ElfWorkshop {
        elfs: Vec<Address>,
        // Maps elf's badge to an hashmap mapping toy name to quantity
        toys: HashMap<Address, HashMap<String, u32>>
    }

    impl ElfWorkshop {
        pub fn new() -> Component {
            // Instantiate the component
            Self {
                elfs: Vec::new(),
                toys: HashMap::new()
            }
            .instantiate()
        }

        pub fn become_elf(&mut self) -> Bucket {
            info!("Welcome to the factory, here is your badge");

            // Create a new badge and send it to the caller
            let elf_badge: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Elf Badge")
                .initial_supply_fungible(1);

            self.elfs.push(elf_badge.resource_address());

            elf_badge
        }

        pub fn create_toy(&mut self, name: String, badge: BucketRef) {
            assert!(badge.amount() > Decimal::zero(), "Where is your badge ?");
            assert!(self.elfs.contains(&badge.resource_address()), "That's not a valid bage !");
            
            // The badge's address is used to identify the elf
            let elf_id = badge.resource_address();

            // We always need to drop bucket refs or else we get an error !
            badge.drop();

            // Insert the toy in the hashmap
            let elf_toys = self.toys.entry(elf_id).or_insert(HashMap::new());
            elf_toys.entry(name.clone()).and_modify(|e| { *e += 1 }).or_insert(1);
        }

        // View the amount of toys you created
        pub fn view_created_toys(&mut self, badge: BucketRef) {
            assert!(badge.amount() > Decimal::zero(), "You have to provide a badge");
            assert!(self.elfs.contains(&badge.resource_address()), "That's not a valid bage !");
            let elf_id = badge.resource_address();

            badge.drop();

            let elf_toys = self.toys.get(&elf_id).unwrap();

            for (toy, amount) in elf_toys.iter() {
                info!("{} created {} times", toy, amount);
            }
        }
    }
}

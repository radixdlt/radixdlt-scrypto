use scrypto::prelude::*;

// ElfWorkshop component. 
// People can register as elf to receive a badge.
// They can then use the badge to create new toys and the component
// keeps track of the amount of toys each elf created.
blueprint! {
    struct ElfWorkshop {
        // Vault that will contain the badge allowing this component to mint new elf_badges
        elf_badge_minter: Vault,
        // Resource definition of the elf badges
        elf_badge: ResourceDef,
        // Maps elf's badge to an hashmap mapping toy name to quantity
        toys: HashMap<Address, HashMap<String, u32>>
    }

    impl ElfWorkshop {
        pub fn new() -> Component {

            // Create a badge allowing this component to mint new elf badges
            let elf_badge_minter: Bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Elf badge minter")
                .initial_supply_fungible(1);

            // Define a mutable resource representing the elf badges
            let elf_badges: ResourceDef = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", "Elf Badge")
                .flags(MINTABLE)
                .badge(elf_badge_minter.resource_address(), MAY_MINT)
                .no_initial_supply();

            // Instantiate the component
            Self {
                elf_badge_minter: Vault::with_bucket(elf_badge_minter),
                elf_badge: elf_badges,
                toys: HashMap::new()
            }
            .instantiate()
        }

        pub fn become_elf(&mut self) -> Bucket {
            info!("Welcome to the factory, here is your badge");

            // Mint a new badge and send it to the caller
            // Vault.authorize is a shortcut, instead of having to take the badge from 
            // the vault and putting it back in after.
            self.elf_badge_minter.authorize(|badge| {
                self.elf_badge.mint(1, badge)
            })
        }

        pub fn create_toy(&mut self, name: String, badge: BucketRef) {
            assert!(badge.amount() > Decimal::zero(), "Where is your badge ?");
            assert!(badge.resource_def() == self.elf_badge, "That's not a valid bage !");
            
            // The badge's address is used to identify the elf
            let elf_id = badge.resource_address();

            // We always need to drop bucket refs or else we get an error !
            badge.drop();

            // Insert the toy in the hashmap
            let elf_toys = self.toys.entry(elf_id).or_insert(HashMap::new());
            let old_count = *elf_toys.entry(name.clone()).or_insert(0);
            elf_toys.insert(name.clone(), old_count + 1);

            info!("The total amount of {} you created is {}", name, old_count + 1)
        }
        
    }
}

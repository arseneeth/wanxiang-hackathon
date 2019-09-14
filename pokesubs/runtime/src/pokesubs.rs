use support::{decl_storage, decl_module, StorageValue, StorageMap,
    dispatch::Result, ensure, decl_event, traits::Currency};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash, Zero};
use parity_codec::{Encode, Decode};
use rstd::cmp;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Poke<Hash, Balance> {
    id: Hash,
    dna: Hash,
    price: Balance,
    gen: u64,
}

pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
        <T as balances::Trait>::Balance
    {
        Created(AccountId, Hash),
        PriceSet(AccountId, Hash, Balance),
        Transferred(AccountId, AccountId, Hash),
        Bought(AccountId, AccountId, Hash, Balance),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as PokeStorage {
        Pokes get(poke): map T::Hash => Poke<T::Hash, T::Balance>;
        PokeOwner get(owner_of): map T::Hash => Option<T::AccountId>;

        AllPokeArray get(poke_by_index): map u64 => T::Hash;
        AllPokeCount get(all_poke_count): u64;
        AllPokeIndex: map T::Hash => u64;

        OwnedPokeArray get(poke_of_owner_by_index): map (T::AccountId, u64) => T::Hash;
        OwnedPokeCount get(owned_poke_count): map T::AccountId => u64;
        OwnedPokeIndex: map T::Hash => u64;
        
        Nonce: u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        fn create_poke(origin) -> Result {
            let sender = ensure_signed(origin)?;
            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            let new_poke = Poke {
                id: random_hash,
                dna: random_hash,
                price: <T::Balance as As<u64>>::sa(0),
                gen: 0,
            };

            Self::mint(sender, random_hash, new_poke)?;
            
            <Nonce<T>>::mutate(|n| *n += 1);

            Ok(())
        }

        fn set_price(origin, poke_id: T::Hash, new_price: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<Pokes<T>>::exists(poke_id), "This cat does not exist");

            let owner = Self::owner_of(poke_id).ok_or("No owner for this pokemon")?;
            ensure!(owner == sender, "You do not own this cat");

            let mut poke = Self::poke(poke_id);
            poke.price = new_price;

            <Pokes<T>>::insert(poke_id, poke);

            Self::deposit_event(RawEvent::PriceSet(sender, poke_id, new_price));

            Ok(())
        }
        
        fn transfer(origin, to: T::AccountId, poke_id: T::Hash) -> Result {
            let sender = ensure_signed(origin)?;

            let owner = Self::owner_of(poke_id).ok_or("No owner for this poke")?;
            ensure!(owner == sender, "You do not own this poke");

            Self::transfer_from(sender, to, poke_id)?;

            Ok(())
        }

        fn buy_poke(origin, poke_id: T::Hash, max_price: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(<Pokes<T>>::exists(poke_id), "This cat does not exist");

            let owner = Self::owner_of(poke_id).ok_or("No owner for this pokemon")?;
            ensure!(owner != sender, "You can't buy your own cat");

            let mut poke = Self::poke(poke_id);

            let poke_price = poke.price;
            ensure!(!poke_price.is_zero(), "The pokemon you want to buy is not for sale");
            ensure!(poke_price <= max_price, "The pokemon you want to buy costs more than your max price");

            <balances::Module<T> as Currency<_>>::transfer(&sender, &owner, poke_price)?;

            Self::transfer_from(owner.clone(), sender.clone(), poke_id)
                .expect("`owner` is shown to own the pokemon; \
                `owner` must have greater than 0 pokemon, so transfer cannot cause underflow; \
                `all_poke_count` shares the same type as `owned_poke_count` \
                and minting ensure there won't ever be more than `max()` kitties, \
                which means transfer cannot cause an overflow; \
                qed");

            poke.price = <T::Balance as As<u64>>::sa(0);
            <Pokes<T>>::insert(poke_id, poke);

            Self::deposit_event(RawEvent::Bought(sender, owner, poke_id, poke_price));

            Ok(())
        }

        fn breed_poke(origin, poke_id_1: T::Hash, poke_id_2: T::Hash) -> Result{
            let sender = ensure_signed(origin)?;

            ensure!(<Pokes<T>>::exists(poke_id_1), "This pokemon 1 does not exist");
            ensure!(<Pokes<T>>::exists(poke_id_2), "This pokemon 2 does not exist");

            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            let poke_1 = Self::poke(poke_id_1);
            let poke_2 = Self::poke(poke_id_2);

            let mut final_dna = poke_1.dna;
            for (i, (dna_2_element, r)) in poke_2.dna.as_ref().iter().zip(random_hash.as_ref().iter()).enumerate() {
                if r % 2 == 0 {
                    final_dna.as_mut()[i] = *dna_2_element;
                }
            }

            let new_poke = Poke {
                id: random_hash,
                dna: final_dna,
                price: <T::Balance as As<u64>>::sa(0),
                gen: cmp::max(poke_1.gen, poke_2.gen) + 1,
            };

            Self::mint(sender, random_hash, new_poke)?;

            <Nonce<T>>::mutate(|n| *n += 1);

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn mint(to: T::AccountId, poke_id: T::Hash, new_poke: Poke<T::Hash, T::Balance>) -> Result {
        ensure!(!<PokeOwner<T>>::exists(poke_id), "Pokemon already exists");

        let owned_poke_count = Self::owned_poke_count(&to);

        let new_owned_poke_count = owned_poke_count.checked_add(1)
            .ok_or("Overflow adding a new pokemon to account balance")?;

        let all_poke_count = Self::all_poke_count();

        let new_all_poke_count = all_poke_count.checked_add(1)
            .ok_or("Overflow adding a new poke to total supply")?;

        <Pokes<T>>::insert(poke_id, new_poke);
        <PokeOwner<T>>::insert(poke_id, &to);

        <AllPokeArray<T>>::insert(all_poke_count, poke_id);
        <AllPokeCount<T>>::put(new_all_poke_count);
        <AllPokeIndex<T>>::insert(poke_id, all_poke_count);

        <OwnedPokeArray<T>>::insert((to.clone(), owned_poke_count), poke_id);
        <OwnedPokeCount<T>>::insert(&to, new_owned_poke_count);
        <OwnedPokeIndex<T>>::insert(poke_id, owned_poke_count);

        Self::deposit_event(RawEvent::Created(to, poke_id));

        Ok(())
    }

    fn transfer_from(from: T::AccountId, to: T::AccountId, poke_id: T::Hash) -> Result {
        let owner = Self::owner_of(poke_id).ok_or("No owner for this pokemon")?;

        ensure!(owner == from, "'from' account does not own this pokemon");

        let owned_poke_count_from = Self::owned_poke_count(&from);
        let owned_poke_count_to = Self::owned_poke_count(&to);

        let new_owned_poke_count_to = owned_poke_count_to.checked_add(1)
            .ok_or("Transfer causes overflow of 'to' pokemon balance")?;

        let new_owned_poke_count_from = owned_poke_count_from.checked_sub(1)
            .ok_or("Transfer causes underflow of 'from' pokemon balance")?;

        let poke_index = <OwnedPokeIndex<T>>::get(poke_id);
        if poke_index != new_owned_poke_count_from {
            let last_poke_id = <OwnedPokeArray<T>>::get((from.clone(), new_owned_poke_count_from));
            <OwnedPokeArray<T>>::insert((from.clone(), poke_index), last_poke_id);
            <OwnedPokeIndex<T>>::insert(last_poke_id, poke_index);
        }

        <PokeOwner<T>>::insert(&poke_id, &to);
        <OwnedPokeIndex<T>>::insert(poke_id, owned_poke_count_to);

        <OwnedPokeArray<T>>::remove((from.clone(), new_owned_poke_count_from));
        <OwnedPokeArray<T>>::insert((to.clone(), owned_poke_count_to), poke_id);

        <OwnedPokeCount<T>>::insert(&from, new_owned_poke_count_from);
        <OwnedPokeCount<T>>::insert(&to, new_owned_poke_count_to);
        
        Self::deposit_event(RawEvent::Transferred(from, to, poke_id));
        
        Ok(())
    }
}

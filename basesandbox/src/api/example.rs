/// First, you have to declare your global context
#[global_context]
struct Context {
    //...
}

/// 4 categories of Handle traits

type Transaction = u8;
/// 1. HostCallBack
/// - You implement it, and never call it.
/// - Host might call methods anytime
/// - There will be only one trait
trait HostCallBack {
    fn excute_transaction(tx: Transaction);
}

/// 2. HostCallable
/// - You call it, and never implement it
/// - Host will provide this.
/// - There will be only one trait
trait HostApi {
    fn query_something();
}

/// 3. ApplicationCallBack
/// - You implement it, and never call it
/// - Other applications call methods anytime
/// - There might be multiple traits
trait HandleToBeGivenToStakingModule {
    fn get_balance() -> u64;
}

/// 4. ApplicationCallable
/// - You call it, and never implement it
/// - Application will provide this
/// - There might be multiple traits
trait HandleToBeGivenByStakingModule {
    fn get_delegation() -> u64;
}

/// Usage
/// Macro will generate corressponding struct for each trait
/// To find such identifier of each, there will be another macro to retrieve that
/// For 2 Callbacks, they will utilize module's global context
///
///
///
///a

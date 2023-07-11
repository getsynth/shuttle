pub mod a_creating;
pub mod b_attaching;
pub mod c_starting;
pub mod d_started;
pub mod e_readying;
pub mod f_ready;
pub mod f_running;
pub mod g_completed;
pub mod g_rebooting;
pub mod h_recreating;
pub mod i_restarting;
pub mod j_stopped;
pub mod k_stopping;
pub mod l_destroying;
pub mod m_destroyed;
pub mod m_errored;
pub mod machine;

pub trait StateVariant {
    fn name() -> String;
    fn as_state_variant(&self) -> String {
        Self::name()
    }
}

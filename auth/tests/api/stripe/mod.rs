use self::{
    active_subscription::MOCKED_ACTIVE_SUBSCRIPTION,
    cancelledpro_checkout_session::MOCKED_CANCELLEDPRO_CHECKOUT_SESSION,
    cancelledpro_subscription_active::MOCKED_CANCELLEDPRO_SUBSCRIPTION_ACTIVE,
    cancelledpro_subscription_cancelled::MOCKED_CANCELLEDPRO_SUBSCRIPTION_CANCELLED,
    completed_checkout_session::MOCKED_COMPLETED_CHECKOUT_SESSION,
    incomplete_checkout_session::MOCKED_INCOMPLETE_CHECKOUT_SESSION,
    overdue_payment_checkout_session::MOCKED_OVERDUE_PAYMENT_CHECKOUT_SESSION,
    past_due_subscription::MOCKED_PAST_DUE_SUBSCRIPTION,
};

mod active_subscription;
mod cancelledpro_checkout_session;
mod cancelledpro_subscription_active;
mod cancelledpro_subscription_cancelled;
mod completed_checkout_session;
mod incomplete_checkout_session;
mod overdue_payment_checkout_session;
mod past_due_subscription;

pub(crate) const MOCKED_SUBSCRIPTIONS: &[&str] = &[
    MOCKED_ACTIVE_SUBSCRIPTION,
    MOCKED_PAST_DUE_SUBSCRIPTION,
    MOCKED_CANCELLEDPRO_SUBSCRIPTION_ACTIVE,
    MOCKED_CANCELLEDPRO_SUBSCRIPTION_CANCELLED,
];

pub(crate) const MOCKED_CHECKOUT_SESSIONS: &[&str] = &[
    MOCKED_COMPLETED_CHECKOUT_SESSION,
    MOCKED_INCOMPLETE_CHECKOUT_SESSION,
    MOCKED_OVERDUE_PAYMENT_CHECKOUT_SESSION,
    MOCKED_CANCELLEDPRO_CHECKOUT_SESSION,
];

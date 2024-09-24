use crate::database::Database;

const _: () = {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    // RFC 2056
    fn assert_all() {
        assert_send::<Database>();
        assert_sync::<Database>();
    }
};

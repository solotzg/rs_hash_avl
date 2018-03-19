macro_rules! offset_of {
    ($TYPE: ty, $MEMBER: ident) => {&(*(0 as *const $TYPE)).$MEMBER as *const _ as isize};
    ($TYPE: ty, $MEMBER: tt) => {&(*(0 as *const $TYPE)).$MEMBER as *const _ as isize};
}
macro_rules! container_of {
    ($PTR: expr, $TYPE: ty, $MEMBER: ident) => {($PTR as *const _ as isize - unsafe {offset_of!($TYPE, $MEMBER)}) as *mut $TYPE};
    ($PTR: expr, $TYPE: ty, $MEMBER: tt) => {($PTR as *const _ as isize - unsafe {offset_of!($TYPE, $MEMBER)}) as *mut $TYPE};
}
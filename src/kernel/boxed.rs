use core::marker::{PhantomData, Unsize};
use core::ptr;
use core::mem::size_of;

pub struct Box<T>
    where T: ?Sized,
{
    _data: PhantomData<T>,
}
impl<T> Box<T>
    where T: ?Sized,
{
    pub fn new<V>(val: V, memory: &mut [u8]) -> &'static mut T
        where V: 'static + Unsize<T>,
    {
        unsafe {
            // todo: select to put variable at beginning or end!
            // copy to static memory
            let offset = (memory.len() - size_of::<V>()) as isize;
            ptr::write(memory.as_mut_ptr().offset(offset) as *mut _, val);
            // create trait object referencing to static memory
            let mut bla_ptr = memory.as_mut_ptr().offset(offset) as *mut V;
            &mut (*bla_ptr) as &mut T
        }
    }
}
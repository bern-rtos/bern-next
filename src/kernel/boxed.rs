use core::marker::{PhantomData, Unsize};
use core::ptr;

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
            // copy to static memory
            ptr::write(memory.as_mut_ptr() as *mut _, val);
            // create trait object referencing to static memory
            let mut bla_ptr = memory.as_mut_ptr() as *mut V;
            &mut (*bla_ptr) as &mut T
        }
    }
}

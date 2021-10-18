//! Functions for dealing with the parallel thread execution model employed by CUDA.
//!
//! # CUDA Thread model
//!
//! The CUDA thread model is based on 3 main structures:
//! - Threads
//! - Thread Blocks
//! - Grids
//!
//! ## Threads
//!
//! Threads are the fundamental element of GPU computing. Threads execute the same kernel
//! at the same time, controlling their task by retrieving their corresponding global thread ID.
//!
//! # Thread Blocks
//!
//! The most important structure after threads, thread blocks arrange

// TODO: write some docs about the terms used in this module.

use cuda_std_macros::gpu_only;
use vek::{Vec2, Vec3};

// different calling conventions dont exist in nvptx, so we just use C as a placeholder.
extern "C" {
    // defined in libintrinsics.ll
    fn __nvvm_thread_idx_x() -> u32;
    fn __nvvm_thread_idx_y() -> u32;
    fn __nvvm_thread_idx_z() -> u32;

    fn __nvvm_block_dim_x() -> u32;
    fn __nvvm_block_dim_y() -> u32;
    fn __nvvm_block_dim_z() -> u32;

    fn __nvvm_block_idx_x() -> u32;
    fn __nvvm_block_idx_y() -> u32;
    fn __nvvm_block_idx_z() -> u32;

    fn __nvvm_grid_dim_x() -> u32;
    fn __nvvm_grid_dim_y() -> u32;
    fn __nvvm_grid_dim_z() -> u32;

    fn __nvvm_warp_size() -> u32;

    fn __nvvm_block_barrier();

    fn __nvvm_grid_fence();
    fn __nvvm_device_fence();
    fn __nvvm_system_fence();
}

#[inline(always)]
pub fn thread_idx_x() -> usize {
    unsafe { __nvvm_thread_idx_x() as usize }
}

#[inline(always)]
pub fn thread_idx_y() -> usize {
    unsafe { __nvvm_thread_idx_y() as usize }
}

#[inline(always)]
pub fn thread_idx_z() -> usize {
    unsafe { __nvvm_thread_idx_z() as usize }
}

#[inline(always)]
pub fn block_idx_x() -> usize {
    unsafe { __nvvm_block_idx_x() as usize }
}

#[inline(always)]
pub fn block_idx_y() -> usize {
    unsafe { __nvvm_block_idx_y() as usize }
}

#[inline(always)]
pub fn block_idx_z() -> usize {
    unsafe { __nvvm_block_idx_z() as usize }
}

#[inline(always)]
pub fn block_dim_x() -> usize {
    unsafe { __nvvm_block_dim_x() as usize }
}

#[inline(always)]
pub fn block_dim_y() -> usize {
    unsafe { __nvvm_block_dim_y() as usize }
}

#[inline(always)]
pub fn block_dim_z() -> usize {
    unsafe { __nvvm_block_dim_z() as usize }
}

#[inline(always)]
pub fn grid_dim_x() -> usize {
    unsafe { __nvvm_grid_dim_x() as usize }
}

#[inline(always)]
pub fn grid_dim_y() -> usize {
    unsafe { __nvvm_grid_dim_y() as usize }
}

#[inline(always)]
pub fn grid_dim_z() -> usize {
    unsafe { __nvvm_grid_dim_z() as usize }
}

/// Gets the 3d index of the thread currently executing the kernel.
#[inline(always)]
pub fn thread_idx() -> Vec3<usize> {
    unsafe {
        Vec3::new(
            __nvvm_thread_idx_x() as usize,
            __nvvm_thread_idx_y() as usize,
            __nvvm_thread_idx_z() as usize,
        )
    }
}

/// Gets the 3d index of the block that the thread currently executing the kernel is located in.
#[inline(always)]
pub fn block_idx() -> Vec3<usize> {
    unsafe {
        Vec3::new(
            __nvvm_block_idx_x() as usize,
            __nvvm_block_idx_y() as usize,
            __nvvm_block_idx_z() as usize,
        )
    }
}

/// Gets the 3d layout of the thread blocks executing this kernel. In other words,
/// how many threads exist in each thread block in every direction.
#[inline(always)]
pub fn block_dim() -> Vec3<usize> {
    unsafe {
        Vec3::new(
            __nvvm_block_dim_x() as usize,
            __nvvm_block_dim_y() as usize,
            __nvvm_block_dim_z() as usize,
        )
    }
}

/// Gets the 3d layout of the block grids executing this kernel. In other words,
/// how many thread blocks exist in each grid in every direction.
#[inline(always)]
pub fn grid_dim() -> Vec3<usize> {
    unsafe {
        Vec3::new(
            __nvvm_grid_dim_x() as usize,
            __nvvm_grid_dim_y() as usize,
            __nvvm_grid_dim_z() as usize,
        )
    }
}

/// Gets the overall thread index, accounting for 1d/2d/3d block/grid dimensions. This
/// value is most commonly used for indexing into data and this index is guaranteed to
/// be unique for every single thread executing this kernel no matter the launch configuration.
/// 
/// For very simple kernels it may be faster to use a more simple index calculation, however,
/// it will be unsound if the kernel launches in a 2d/3d configuration.
#[rustfmt::skip]
#[inline(always)]
pub fn index() -> usize {
    let grid_dim = grid_dim();
    let block_idx = block_idx();
    let block_dim = block_dim();
    let thread_idx = thread_idx();

    let block_id = block_idx.x + block_idx.y * grid_dim.x 
                       + grid_dim.x * grid_dim.y * block_idx.z;

    block_id * block_dim.product()
    + (thread_idx.z * (block_dim.x * block_dim.y))
    + (thread_idx.y * block_dim.x) + thread_idx.x
}

#[inline(always)]
pub fn index_2d() -> Vec2<usize> {
    let i = thread_idx_x() + block_idx_x() * block_dim_x();
    let j = thread_idx_y() + block_idx_y() * block_dim_y();
    Vec2::new(i, j)
}

#[inline(always)]
pub fn index_3d() -> Vec3<usize> {
    let i = thread_idx_x() + block_idx_x() * block_dim_x();
    let j = thread_idx_y() + block_idx_y() * block_dim_y();
    let k = thread_idx_z() + block_idx_z() * block_dim_z();
    Vec3::new(i, j, k)
}

/// Whether this is the first thread (not the first thread to be executing). This function is guaranteed
/// to only return true in a single thread that is invoking it. This is useful for only doing something
/// once.
#[inline(always)]
pub fn first() -> bool {
    block_idx() == Vec3::zero() && thread_idx() == Vec3::zero()
}

/// Gets the number of threads inside of a warp. Currently 32 threads on every GPU architecture.
#[inline(always)]
pub fn warp_size() -> usize {
    unsafe { __nvvm_warp_size() as usize }
}

/// Waits until all threads in the thread block have reached this point. This guarantees
/// that any global or shared mem accesses are visible to every thread after this call.
///
/// Be careful when using sync_threads in conditional code. It will be perfectly fine if
/// all threads evaluate to the same path, but if they dont, execution will halt
/// or produce odd results (but should not produce undefined behavior).
#[inline(always)]
pub fn sync_threads() {
    unsafe { __nvvm_block_barrier() }
}

/// Identical to [`sync_threads`] but with the additional feature that it evaluates
/// the predicate for every thread and returns the number of threads in which it evaluated to a non-zero number.
#[gpu_only]
#[inline(always)]
pub fn sync_threads_count(predicate: u32) -> u32 {
    extern "C" {
        #[link_name = "llvm.nvvm.barrier0.popc"]
        fn __nvvm_sync_threads_count(predicate: u32) -> u32;
    }

    unsafe { __nvvm_sync_threads_count(predicate) }
}

/// Identical to [`sync_threads`] but with the additional feature that it evaluates
/// the predicate for every thread and returns a non-zero integer if every predicate evaluates to non-zero for all threads.
#[gpu_only]
#[inline(always)]
pub fn sync_threads_and(predicate: u32) -> u32 {
    extern "C" {
        #[link_name = "llvm.nvvm.barrier0.and"]
        fn __nvvm_sync_threads_and(predicate: u32) -> u32;
    }

    unsafe { __nvvm_sync_threads_and(predicate) }
}

/// Identical to [`sync_threads`] but with the additional feature that it evaluates
/// the predicate for every thread and returns a non-zero integer if at least one predicate in a thread evaluates
/// to non-zero.
#[gpu_only]
#[inline(always)]
pub fn sync_threads_or(predicate: u32) -> u32 {
    extern "C" {
        #[link_name = "llvm.nvvm.barrier0.or"]
        fn __nvvm_sync_threads_or(predicate: u32) -> u32;
    }

    unsafe { __nvvm_sync_threads_or(predicate) }
}

/// Acts as a memory fence at the grid level (all threads inside of a kernel execution).
///
/// Note that this is NOT an execution synchronization like [`sync_threads`]. It is not possible
/// to sync threads at a grid level. It is simply a memory fence.
#[inline(always)]
pub fn grid_fence() {
    unsafe { __nvvm_grid_fence() }
}

/// Acts as a memory fence at the device level.
#[inline(always)]
pub fn device_fence() {
    unsafe { __nvvm_device_fence() }
}

/// Acts as a memory fence at the system level.
#[inline(always)]
pub fn system_fence() {
    unsafe { __nvvm_system_fence() }
}

/// Suspends the calling thread for a duration (in nanoseconds) approximately close to `nanos`.
///
/// This is useful for implementing something like a mutex with exponential back-off.
#[gpu_only]
#[inline(always)]
pub fn nanosleep(nanos: u32) {
    unsafe {
        asm!(
            "nanosleep {}",
            in(reg32) nanos
        )
    }
}
//! Types related to task management

use super::TaskContext;
use alloc::vec::Vec;

/// The task control block (TCB) of a task.
#[derive(Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// Syscall count statistics: (syscall_id, count) pairs
    pub syscall_counts: Vec<(usize, usize)>,
}

impl TaskControlBlock {
    /// Create a new TaskControlBlock
    pub fn new(task_cx: TaskContext, task_status: TaskStatus) -> Self {
        Self {
            task_status,
            task_cx,
            syscall_counts: Vec::new(),
        }
    }

    /// Increment syscall count for given syscall_id
    pub fn increment_syscall_count(&mut self, syscall_id: usize) {
        for (id, count) in &mut self.syscall_counts {
            if *id == syscall_id {
                *count += 1;
                return;
            }
        }
        // If syscall_id not found, add new entry
        self.syscall_counts.push((syscall_id, 1));
    }

    /// Get syscall count for given syscall_id
    pub fn get_syscall_count(&self, syscall_id: usize) -> usize {
        for (id, count) in &self.syscall_counts {
            if *id == syscall_id {
                return *count;
            }
        }
        0
    }
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}

#![allow(dead_code)]
/// 
/// Copyright 2023, [object Object]
/// Licensed under MIT
///

use crate::Iteration;

pub struct SortableArray<'a> {
    pub arr: &'a mut [u32],
    iterations: Vec<Iteration>,
}

impl<'a> SortableArray<'a> {
    pub fn new(arr: &'a mut [u32]) -> Self {
        Self {
            arr,
            iterations: Vec::new(),
        }
    }

    // merge sort implementation
    fn merge(&mut self, l: usize, m: usize, r: usize) {
        let mut left = self.arr[l..=m].to_vec();
        let mut right = self.arr[m + 1..=r].to_vec();

        left.push(u32::MAX);
        right.push(u32::MAX);

        let mut i = 0;
        let mut j = 0;

        for k in l..=r {
            if left[i] <= right[j] {
                self.arr[k] = left[i];
                i += 1;
            } else {
                self.arr[k] = right[j];
                j += 1;
            }
            self.iterations.push(Iteration {
                arr: self.arr.to_vec(),
                index: vec![self.arr[k]],
            });
        }
    }

    pub fn merge_sort_rec(&mut self, l: usize, r: usize) {
        if l < r {
            let m = l + (r - l) / 2;
            self.merge_sort_rec(l, m);
            self.merge_sort_rec(m + 1, r);
            self.merge(l, m, r);
        }
    }

    // quick sort implementation
    pub fn quicksort(&mut self, low: usize, high: usize) {
        if high > low {
            let pivot_index = self.partition(low, high);
            if pivot_index > 0 {
                self.quicksort(low, pivot_index - 1);
            }
            self.quicksort(pivot_index + 1, high);
        }
    }

    fn partition(&mut self, low: usize, high: usize) -> usize {
        let pivot = self.arr[low];
        let mut i = low + 1;
        let mut j = high;
        loop {
            while i <= j && self.arr[i] <= pivot {
                i += 1;
            }
            while i <= j && self.arr[j] > pivot {
                j -= 1;
            }
            if i >= j {
                break;
            }
            self.arr.swap(i, j);
        }
        self.arr.swap(low, j);
        self.iterations.push(Iteration {
            arr: self.arr.to_vec(),
            index: vec![pivot, j as u32],
        });

        j
    }

    // bubble sort implementation
    pub fn bubble_sort(&mut self) {
        for i in 0..self.arr.len() {
            for j in 0..self.arr.len() - i - 1 {
                if self.arr[j] > self.arr[j + 1] {
                    self.arr.swap(j, j + 1);
                }

                self.iterations.push(Iteration {
                    arr: self.arr.to_vec(),
                    index: vec![j as u32, (j + 1) as u32],
                });
            }
        }
    }

    // selection sort implementation
    pub fn selection_sort(&mut self) {
        for i in 0..self.arr.len() {
            let mut min = i;
            for j in i + 1..self.arr.len() {
                if self.arr[j] < self.arr[min] {
                    min = j;
                }
            }
            self.arr.swap(i, min);
            self.iterations.push(Iteration {
                arr: self.arr.to_vec(),
                index: vec![min as u32],
            });
        }
    }

    // insertion sort implementation
    pub fn insertion_sort(&mut self) {
        for i in 1..self.arr.len() {
            let mut j = i;
            while j > 0 && self.arr[j - 1] > self.arr[j] {
                self.arr.swap(j - 1, j);
                j -= 1;
            }
            self.iterations.push(Iteration {
                arr: self.arr.to_vec(),
                index: vec![i as u32, j as u32],
            });
        }
    }

    // heap sort implementation
    fn heapify(&mut self, len: usize, i: usize) {
        let mut largest = i;
        let left = 2 * i + 1;
        let right = 2 * i + 2;
        if left < len && self.arr[left] > self.arr[largest] {
            largest = left;
        }
        if right < len && self.arr[right] > self.arr[largest] {
            largest = right;
        }
        if largest != i {
            self.arr.swap(i, largest);
            self.heapify(len, largest);
        }
        self.iterations.push(Iteration {
            arr: self.arr.to_vec(),
            index: vec![i as u32, largest as u32],
        });
    }

    pub fn heap_sort(&mut self) {
        let len = self.arr.len();
        for i in (0..len / 2).rev() {
            self.heapify(len, i);
        }
        for i in (1..len).rev() {
            self.arr.swap(0, i);
            self.heapify(i, 0);
        }
    }

    // shell sort implementation
    pub fn shell_sort(&mut self) {
        let len = self.arr.len();
        let mut gap = len / 2;
        while gap > 0 {
            for i in gap..len {
                let mut j = i;
                while j >= gap && self.arr[j - gap] > self.arr[j] {
                    self.arr.swap(j - gap, j);
                    j -= gap;
                }
                self.iterations.push(Iteration {
                    arr: self.arr.to_vec(),
                    index: vec![i as u32, j as u32],
                });
            }
            gap /= 2;
        }
    }

    // Bogo sort implementation
    fn is_sorted(&mut self) -> bool {
        for i in 1..self.arr.len() {
            self.iterations.push(Iteration {
                arr: self.arr.to_vec(),
                index: vec![self.arr[i]],
            });

            if self.arr[i] < self.arr[i - 1] {
                return false;
            }
        }
        true
    }

    pub fn bogo_sort(&mut self) {
        use rand::seq::SliceRandom;
        use rand::thread_rng;

        let mut rng = thread_rng();
        while !self.is_sorted() {
            self.arr.shuffle(&mut rng);
        }
    }

    // Cocktail sort implementation
    pub fn cocktail_sort(&mut self) {
        let n = self.arr.len();
        let mut swapped = true;
        let mut start = 0;
        let mut end = n - 1;

        while swapped {
            swapped = false;

            for i in start..end {
                if self.arr[i] > self.arr[i + 1] {
                    self.arr.swap(i, i + 1);
                    swapped = true;
                }
                let tone = if self.arr[i] > self.arr[i + 1] {
                    self.arr[i]
                } else {
                    self.arr[i + 1]
                };

                self.iterations.push(Iteration {
                    arr: self.arr.to_vec(),
                    index: vec![tone],
                });
            }

            if !swapped {
                break;
            }

            swapped = false;
            end -= 1;

            for i in (start..end).rev() {
                if self.arr[i] > self.arr[i + 1] {
                    self.arr.swap(i, i + 1);
                    swapped = true;
                }
                let tone = if self.arr[i] > self.arr[i + 1] {
                    self.arr[i]
                } else {
                    self.arr[i + 1]
                };

                self.iterations.push(Iteration {
                    arr: self.arr.to_vec(),
                    index: vec![tone],
                });
            }

            start += 1;
        }
    }

    pub fn get_iterations(self) -> Vec<Iteration> {
        self.iterations
    }
}

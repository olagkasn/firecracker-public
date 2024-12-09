// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use std::sync::{Arc, Mutex};
use crate::devices::virtio::memory::device::Memory;
use serde::{Deserialize, Serialize};
type MutexMemory = Arc<Mutex<Memory>>;
/// Errors associated with the operations allowed on the memory.

#[derive(Debug, thiserror::Error, displaydoc::Display)]
pub enum MemoryConfigError {
    /// The user made a request on an inexistent memory device.
    DeviceNotFound,
    /// Device not activated yet.
    DeviceNotActive,
    /// There already exists a device with this id.
    DeviceWithThisIdExists,
    /// Failed to create a memory device.
    CreateFailure(crate::devices::virtio::memory::Error),
}

type Result<T> = std::result::Result<T, MemoryConfigError>;
/// This struct represents the strongly typed equivalent of the json body
/// from memory related requests.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryDeviceConfig {
    /// ID of the device.
    pub id: String,
    /// Block size in bytes.
    #[serde(default)]
    pub block_size: u64,
    /// Node id if any.
    #[serde(default)]
    pub node_id: u16,
    /// Region size in bytes.
    pub region_size: u64,
    /// Requested size in bytes.
    #[serde(default)]
    pub requested_size: u64,
}
/// The data fed into a memory update request. The only thing that can be modified
/// is the requested size of the memory region.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryUpdateConfig {
    /// Requested size in bytes.
    pub requested_size: u64,
}
/// A builder for `Memory` devices from 'MemoryDeviceConfig'.
#[derive(Debug)]
pub struct MemoryBuilder {
    memory_devices: Vec<MutexMemory>,
}
// #[cfg(not(test))]
impl Default for MemoryBuilder {
    fn default() -> MemoryBuilder {
        MemoryBuilder {
            memory_devices: Vec::new(),
        }
    }
}
impl MemoryBuilder {
    /// Creates an empty Memory Store.
    pub fn new() -> Self {
        Default::default()
    }
    /// Creates a Memory device from the MemoryDeviceConfig provided
    fn build(cfg: MemoryDeviceConfig) -> Result<MutexMemory> {
        let memory = Memory::new(
            cfg.block_size,
            if cfg.node_id != 0 {
                Some(cfg.node_id)
            } else {
                None
            },
            cfg.region_size,
            cfg.id,
        )
        .map_err(MemoryConfigError::CreateFailure)?;
        Ok(Arc::new(Mutex::new(memory)))
    }
    /// Inserts into the builder the memory device created from the config.
    pub fn insert(&mut self, cfg: MemoryDeviceConfig) -> Result<()> {
        let memory = Self::build(cfg)?;
        self.add_device(memory)?;
        Ok(())
    }
    /// Inserts an existing memory device.
    pub fn add_device(&mut self, memory: MutexMemory) -> Result<()> {
        for device in &self.memory_devices {
            if device.lock().expect("Poisoned lock").id()
                == memory.lock().expect("Poisoned lock").id()
            {
                return Err(MemoryConfigError::DeviceWithThisIdExists);
            }
        }
        self.memory_devices.push(memory);
        Ok(())
    }
    /// Gets an iterator over mutable references
    pub fn iter_mut(&mut self) -> std::slice::IterMut<MutexMemory> {
        self.memory_devices.iter_mut()
    }
    /// Gets an iterator over references
    pub fn iter(&self) -> std::slice::Iter<MutexMemory> {
        self.memory_devices.iter()
    }
}
#[cfg(test)]
pub(crate) mod tests {
    use utils::get_page_size;
    use super::*;
    fn page_size() -> u64 {
        get_page_size().unwrap() as u64
    }
    fn default_config() -> MemoryDeviceConfig {
        MemoryDeviceConfig {
            id: String::from("memory-dev"),
            block_size: page_size(),
            node_id: 0,
            region_size: 8 * page_size(),
            requested_size: 0,
        }
    }
    fn broken_config() -> MemoryDeviceConfig {
        MemoryDeviceConfig {
            id: String::from("broken-config"),
            block_size: page_size() + 1,
            node_id: 0,
            region_size: page_size() + 2,
            requested_size: 0,
        }
    }
    #[test]
    fn test_insert_duplicate() {
        let mut memory_builder = MemoryBuilder::new();
        // adding one memory device should work
        assert!(memory_builder.insert(default_config()).is_ok());
        // adding the second memory device with the same Id should fail
        match memory_builder.insert(default_config()) {
            Err(MemoryConfigError::DeviceWithThisIdExists) => {}
            _ => unreachable!(),
        }
    }
    #[test]
    fn test_insert_broken_config() {
        let mut memory_builder = MemoryBuilder::new();
        // trying to build a memory device from o ill-formed config
        match memory_builder.insert(broken_config()) {
            Err(MemoryConfigError::CreateFailure(_)) => {}
            _ => unreachable!(),
        }
        // adding a valid one should work
        assert!(memory_builder.insert(default_config()).is_ok());
    }
}
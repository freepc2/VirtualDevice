use super::address::{IoNumber, IoType, NetId, PhyAddress, PhysicalIoAddress};
use super::memory::{IoValue, MemoryError, NetworkMemory};

/// Networks are indexed directly by their small, unsigned NetId.
#[derive(Debug, Default)]
pub struct LocalServer {
    networks: Vec<Option<NetworkMemory>>,
}

impl LocalServer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_network(&mut self, net_id: NetId) -> &mut NetworkMemory {
        let index = usize::from(net_id);
        self.networks.resize_with(index + 1, || None);
        self.networks[index].get_or_insert_with(|| NetworkMemory::new(net_id))
    }
    pub fn network(&self, net_id: NetId) -> Option<&NetworkMemory> {
        self.networks.get(usize::from(net_id))?.as_ref()
    }
    pub fn network_mut(&mut self, net_id: NetId) -> Option<&mut NetworkMemory> {
        self.networks.get_mut(usize::from(net_id))?.as_mut()
    }
    pub fn networks(&self) -> impl Iterator<Item = &NetworkMemory> {
        self.networks.iter().flatten()
    }
    pub fn register(&mut self, address: PhyAddress) -> Result<(), MemoryError> {
        address.validate().map_err(MemoryError::InvalidAddress)?;
        self.add_network(address.net_id).register(address)
    }
    /// Stops at the first error; previously successful registrations remain.
    pub fn register_all<I: IntoIterator<Item = PhyAddress>>(
        &mut self,
        addresses: I,
    ) -> Result<(), MemoryError> {
        for address in addresses {
            self.register(address)?;
        }
        Ok(())
    }
    pub fn read_digital(&self, net_id: NetId, number: IoNumber) -> Option<IoValue> {
        self.network(net_id)?.read_digital(number)
    }
    pub fn read_analog(&self, net_id: NetId, number: IoNumber) -> Option<IoValue> {
        self.network(net_id)?.read_analog(number)
    }
    pub fn read_io(&self, net_id: NetId, io_type: IoType, number: IoNumber) -> Option<IoValue> {
        self.network(net_id)?.read_io(io_type, number)
    }
    pub fn write_digital(
        &mut self,
        net_id: NetId,
        number: IoNumber,
        value: bool,
    ) -> Result<(), MemoryError> {
        self.network_mut(net_id)
            .ok_or(MemoryError::NetworkNotFound(net_id))?
            .write_digital(number, value)
    }
    pub fn write_analog(
        &mut self,
        net_id: NetId,
        number: IoNumber,
        value: IoValue,
    ) -> Result<(), MemoryError> {
        self.network_mut(net_id)
            .ok_or(MemoryError::NetworkNotFound(net_id))?
            .write_analog(number, value)
    }
    pub fn write_io(
        &mut self,
        net_id: NetId,
        io_type: IoType,
        number: IoNumber,
        value: IoValue,
    ) -> Result<(), MemoryError> {
        self.network_mut(net_id)
            .ok_or(MemoryError::NetworkNotFound(net_id))?
            .write_io(io_type, number, value)
    }
    /// This key contains only physical location and I/O type, not an I/O number.
    pub fn read_physical(&self, net_id: NetId, key: &PhysicalIoAddress) -> Option<IoValue> {
        self.network(net_id)?.read_physical(key)
    }
    pub fn write_physical(
        &mut self,
        net_id: NetId,
        key: &PhysicalIoAddress,
        value: IoValue,
    ) -> Result<(), MemoryError> {
        self.network_mut(net_id)
            .ok_or(MemoryError::NetworkNotFound(net_id))?
            .write_physical(key, value)
    }
    pub fn read_phy(&self, address: &PhyAddress) -> Option<IoValue> {
        self.read_physical(address.net_id, &address.physical_key())
    }
    pub fn write_phy(&mut self, address: &PhyAddress, value: IoValue) -> Result<(), MemoryError> {
        self.write_physical(address.net_id, &address.physical_key(), value)
    }
    pub fn network_count(&self) -> usize {
        self.networks.iter().flatten().count()
    }
    pub fn contains_network(&self, net_id: NetId) -> bool {
        self.network(net_id).is_some()
    }
    pub fn remove_network(&mut self, net_id: NetId) -> Option<NetworkMemory> {
        self.networks.get_mut(usize::from(net_id))?.take()
    }
}

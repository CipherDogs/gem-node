use anyhow::{anyhow, Result};
use randomx_rs::{RandomXCache, RandomXFlag, RandomXVM};
use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock},
    time::Instant,
};

#[derive(Clone, Debug)]
pub struct RandomXFactory {
    inner: Arc<RwLock<RandomXFactoryInner>>,
}

impl Default for RandomXFactory {
    fn default() -> Self {
        Self::new(2)
    }
}

impl RandomXFactory {
    pub fn new(max_vms: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(RandomXFactoryInner::new(max_vms))),
        }
    }

    pub fn create(&self, key: &[u8]) -> Result<RandomXVMInstance> {
        let res;
        {
            let mut inner = self.inner.write().unwrap();
            res = inner.create(key)?;
        }
        Ok(res)
    }

    pub fn get_count(&self) -> usize {
        let inner = self.inner.read().unwrap();
        inner.get_count()
    }

    pub fn get_flags(&self) -> RandomXFlag {
        let inner = self.inner.read().unwrap();
        inner.get_flags()
    }
}

struct RandomXFactoryInner {
    flags: RandomXFlag,
    vms: HashMap<Vec<u8>, (Instant, RandomXVMInstance)>,
    max_vms: usize,
}

impl RandomXFactoryInner {
    pub fn new(max_vms: usize) -> Self {
        let flags = RandomXFlag::get_recommended_flags();

        log::trace!(
            "RandomX factory started with {max_vms} max VMs and recommended flags = {flags:?}"
        );

        Self {
            flags,
            vms: Default::default(),
            max_vms,
        }
    }

    pub fn create(&mut self, key: &[u8]) -> Result<RandomXVMInstance> {
        if let Some(entry) = self.vms.get_mut(key) {
            let vm = entry.1.clone();
            entry.0 = Instant::now();
            return Ok(vm);
        }

        if self.vms.len() >= self.max_vms {
            let mut oldest_value = Instant::now();
            let mut oldest_key = None;
            for (k, v) in &self.vms {
                if v.0 < oldest_value {
                    oldest_key = Some(k.clone());
                    oldest_value = v.0;
                }
            }
            if let Some(k) = oldest_key {
                self.vms.remove(&k);
            }
        }

        let vm = RandomXVMInstance::create(key, self.flags)?;

        self.vms
            .insert(Vec::from(key), (Instant::now(), vm.clone()));

        Ok(vm)
    }

    pub fn get_count(&self) -> usize {
        self.vms.len()
    }

    pub fn get_flags(&self) -> RandomXFlag {
        self.flags
    }
}

impl fmt::Debug for RandomXFactoryInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RandomXFactory")
            .field("flags", &self.flags)
            .field("max_vms", &self.max_vms)
            .finish()
    }
}

#[derive(Clone)]
pub struct RandomXVMInstance {
    instance: Arc<RandomXVM>,
}

impl RandomXVMInstance {
    fn create(key: &[u8], flags: RandomXFlag) -> Result<Self> {
        let (flags, cache) = match RandomXCache::new(flags, key) {
            Ok(cache) => (flags, cache),
            Err(error) => {
                log::warn!("Error initializing RandomX cache with flags: {error:?}. Fallback to default flags");

                let flags = RandomXFlag::FLAG_DEFAULT;
                let cache = RandomXCache::new(flags, key).map_err(|error| {
                    anyhow!("Error initializing RandomX cache with default flags: {error:?}")
                })?;

                (flags, cache)
            }
        };

        let vm = RandomXVM::new(flags, Some(cache), None)
            .map_err(|error| anyhow!("Error initializing RandomX VM: {error:?}"))?;

        Ok(Self {
            instance: Arc::new(vm),
        })
    }

    pub fn calculate_hash(&self, input: &[u8]) -> Result<Vec<u8>> {
        self.instance
            .calculate_hash(input)
            .map_err(|error| anyhow!("Hash calculation failed: {error:?}"))
    }
}

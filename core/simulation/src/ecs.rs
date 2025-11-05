use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

pub struct ComponentStorage<T: 'static> {
    components: HashMap<Entity, T>,
}

impl<T: 'static> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, component: T) {
        self.components.insert(entity, component);
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.components.get(&entity)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.components.get_mut(&entity)
    }

    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        self.components.remove(&entity)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.components.iter().map(|(e, c)| (*e, c))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        self.components.iter_mut().map(|(e, c)| (*e, c))
    }
}

impl<T: 'static> Default for ComponentStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct World {
    next_entity_id: u32,
    component_storages: HashMap<TypeId, Box<dyn Any>>,
    resources: HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            component_storages: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity(self.next_entity_id);
        self.next_entity_id += 1;
        entity
    }

    // TODO: think of a deletion approach that won't leak
    // pub fn delete_entity(&mut self, entity: Entity)

    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();

        let storage = self
            .component_storages
            .entry(type_id)
            .or_insert_with(|| Box::new(ComponentStorage::<T>::new()))
            .downcast_mut::<ComponentStorage<T>>()
            .unwrap();

        storage.insert(entity, component);
    }

    pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.component_storages
            .get_mut(&type_id)?
            .downcast_mut::<ComponentStorage<T>>()?
            .remove(entity)
    }

    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.component_storages
            .get(&type_id)?
            .downcast_ref::<ComponentStorage<T>>()?
            .get(entity)
    }

    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.component_storages
            .get_mut(&type_id)?
            .downcast_mut::<ComponentStorage<T>>()?
            .get_mut(entity)
    }

    pub fn query<T: 'static>(&self) -> Vec<(Entity, &T)> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self
            .component_storages
            .get(&type_id)
            .and_then(|s| s.downcast_ref::<ComponentStorage<T>>())
        {
            storage.iter().collect()
        } else {
            Vec::new()
        }
    }

    pub fn query_mut<T: 'static>(&mut self) -> Vec<(Entity, &mut T)> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self
            .component_storages
            .get_mut(&type_id)
            .and_then(|s| s.downcast_mut::<ComponentStorage<T>>())
        {
            storage.iter_mut().collect()
        } else {
            Vec::new()
        }
    }

    /// Insert a global resource (singleton data)
    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.resources.insert(type_id, Box::new(resource));
    }

    /// Get immutable reference to a resource
    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.resources.get(&type_id)?.downcast_ref::<T>()
    }

    /// Get mutable reference to a resource
    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.resources.get_mut(&type_id)?.downcast_mut::<T>()
    }

    /// Remove a resource
    pub fn remove_resource<T: 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.resources
            .remove(&type_id)?
            .downcast::<T>()
            .ok()
            .map(|boxed| *boxed)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

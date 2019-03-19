use crate::worker::{
    component::Component,
    component::ComponentId,
    entity::Entity,
    internal::schema::{
        SchemaComponentData, SchemaDouble, SchemaObject, SchemaObjectField, SchemaPrimitiveField,
        SchemaString, SchemaStringField, SchemaUint32,
    },
};
use std::collections::HashMap;

const POSITION_COMPONENT_ID: ComponentId = 54;
const ENTITY_ACL_COMPONENT_ID: ComponentId = 50;

pub struct EntityBuilder {
    position: (f64, f64, f64),
    entity: Entity,
    write_permissions: HashMap<ComponentId, String>,
    read_permissions: Vec<String>,
    error: Option<String>,
}

impl EntityBuilder {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        EntityBuilder {
            position: (x, y, z),
            entity: Entity::new(),
            write_permissions: HashMap::new(),
            read_permissions: Vec::new(),
            error: None,
        }
    }

    pub fn add_component<C: Component, T: Into<String>>(mut self, data: C, write_layer: T) -> Self {
        if let Err(e) = self.entity.add(data) {
            self.error = Some(e);
        };

        self.write_permissions.insert(C::ID, write_layer.into());
        self
    }

    pub fn set_read_access<T: AsRef<str>>(mut self, layers: &[T]) -> Self {
        self.read_permissions = layers
            .iter()
            .map(|layer| layer.as_ref().to_owned())
            .collect();

        self
    }

    pub fn build(mut self) -> Result<Entity, String> {
        if let Some(e) = self.error {
            return Err(e);
        }

        unsafe { self.entity.add_serialized(self.serialize_position())? };
        unsafe { self.entity.add_serialized(self.serialize_acl())? }

        Ok(self.entity)
    }

    fn serialize_position(&self) -> SchemaComponentData {
        let mut position_schema = SchemaComponentData::new(POSITION_COMPONENT_ID);
        let mut position_fields = position_schema.fields_mut();

        let mut coords_obj = position_fields.field::<SchemaObject>(1).add();
        coords_obj.field::<SchemaDouble>(1).add(self.position.0);
        coords_obj.field::<SchemaDouble>(2).add(self.position.1);
        coords_obj.field::<SchemaDouble>(3).add(self.position.2);

        position_schema
    }

    fn serialize_acl(&self) -> SchemaComponentData {
        let mut acl_schema = SchemaComponentData::new(ENTITY_ACL_COMPONENT_ID);
        let mut acl_fields = acl_schema.fields_mut();

        let mut read_access = acl_fields.field::<SchemaObject>(1).add();
        for layer in &self.read_permissions {
            let mut attribute_set = read_access.field::<SchemaObject>(1).add();
            attribute_set.field::<SchemaString>(1).add(layer);
        }

        for pair in &self.write_permissions {
            let map_obj = acl_fields.field::<SchemaObject>(2).add();
            map_obj.field::<SchemaUint32>(1).add(*pair.0);

            map_obj
                .field::<SchemaObject>(2)
                .add()
                .field::<SchemaString>(1)
                .add(pair.1);
        }

        acl_schema
    }
}
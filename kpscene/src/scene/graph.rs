use crate::scene::*;

pub trait KPSceneGraph {
    fn add_scene(&mut self, scene: &KPScene) -> Result<()>;
}

impl KPSceneGraph for KPGraph {
    fn add_scene(&mut self, scene: &KPScene) -> Result<()> {
        for get_filter in scene.get_filters() {
            self.add_filter(get_filter)?;
        }
        Ok(())
    }
}

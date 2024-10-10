use std::collections::BTreeMap;

pub trait KPPluginItem {
    // The app name of the plugin, and this name needs to be unique within your namespace.
    // ex: text-1
    fn get_name(&self) -> String;

    // Regarding the filter name that need to be used
    fn get_filter_name(&self) -> String;

    // Provide a list of the filter parameters supported by this plugin
    fn default_arguments(&self) -> BTreeMap<String, String>;

    // Provide a list of the user-defined parameters supported by this plugin.
    // These parameters can be overridden by the user to modify default settings.
    fn allow_arguments(&self) -> Vec<String>;

    // This is part of the lifecycle, where "created" typically represents a successfully initialized event.
    fn created(&mut self) -> Result<(), String> {
        Ok(())
    }

    // This is part of the lifecycle, where "mounted" typically represents a notification message after the example is loaded.
    // Please note that this function will be executed every time the media file is switched.
    fn mounted(&mut self) -> Result<(), String> {
        Ok(())
    }

    // This is part of the lifecycle, where "destroy" typically represents a successfully destroy event.
    fn destroy(&mut self) -> Result<(), String> {
        Ok(())
    }
}
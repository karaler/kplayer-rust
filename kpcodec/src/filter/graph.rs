use crate::filter::*;
use crate::filter::filter::KPFilter;

#[derive(Default, Eq, PartialEq, Debug)]
pub enum KPGraphStatus {
    #[default]
    None,
    Initialized,
    Opened,
    Started,
    Stopped,
}

#[derive(Default)]
pub struct KPGraphChain {
    filter: KPFilter,
    filter_context: KPAVFilterContext,
}

#[derive(Default)]
pub struct KPGraph {
    filter_graph: KPAVFilterGraph,
    filter_chain: Vec<Vec<KPGraphChain>>,
    status: KPGraphStatus,
}

impl KPGraph {
    pub fn new() -> Self {
        KPGraph {
            filter_graph: KPAVFilterGraph::new(),
            ..Default::default()
        }
    }

    pub fn injection_source(&mut self) -> Result<()> {
        assert_eq!(self.status, KPGraphStatus::None);
        Ok(())
    }

    pub fn injection_sink(&mut self) -> Result<()> {
        assert_eq!(self.status, KPGraphStatus::Initialized);
        Ok(())
    }

    pub fn add_filter(&mut self, filter: Vec<KPFilter>) -> Result<()> {
        assert_eq!(self.status, KPGraphStatus::Initialized);
        assert!(!self.filter_chain.is_empty());

        // create filter_context
        let mut filter_chains = Vec::new();
        for f in filter.iter() {
            let filter_context = f.create_by_graph(&self.filter_graph)?;
            filter_chains.push(KPGraphChain { filter: f.clone(), filter_context })
        }

        // validate
        let last_filter = self.filter_chain.last().unwrap();
        let output_pads: usize = last_filter.iter().map(|inner| inner.filter_context.get_output_count()).sum();
        let input_pads: usize = filter_chains.iter().map(|inner| inner.filter_context.get_input_count()).sum();
        if output_pads != input_pads {
            return Err(anyhow!("mismatch input and output pads. outputs:{}, inputs:{}", output_pads, input_pads));
        }

        // append filter
        self.filter_chain.push(filter_chains);

        Ok(())
    }
}
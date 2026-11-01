use cosmic::widget::segmented_button;
use std::collections::{BTreeMap, HashMap};

use crate::app::PaneType;
use crate::pane_grid;

type TabModel = segmented_button::Model<segmented_button::SingleSelect>;

pub struct CommanderPaneGrid {
    pub panestates: pane_grid::State<TabModel>,
    pub panes_created: usize,
    pub focus: pane_grid::Pane,
    pub panes: Vec<pane_grid::Pane>,
    pub splits: Vec<pane_grid::Split>,
    pub entity_by_pane: BTreeMap<pane_grid::Pane, segmented_button::Entity>,
    pub entity_by_type: BTreeMap<PaneType, segmented_button::Entity>,
    pub pane_by_entity: BTreeMap<segmented_button::Entity, pane_grid::Pane>,
    pub pane_by_type: BTreeMap<PaneType, pane_grid::Pane>,
    pub type_by_entity: BTreeMap<segmented_button::Entity, PaneType>,
    pub type_by_pane: BTreeMap<pane_grid::Pane, PaneType>,
    pub first_pane: pane_grid::Pane,
}

impl CommanderPaneGrid {
    pub fn new(model: TabModel) -> Self {
        let (panestates, pane) = pane_grid::State::new(model);
        let mut terminal_ids = HashMap::new();
        terminal_ids.insert(pane, cosmic::widget::Id::unique());
        let mut v = Self {
            panestates,
            panes_created: 1,
            focus: pane,
            panes: vec![pane],
            splits: Vec::new(),
            entity_by_pane: BTreeMap::new(),
            entity_by_type: BTreeMap::new(),
            pane_by_entity: BTreeMap::new(),
            pane_by_type: BTreeMap::new(),
            type_by_entity: BTreeMap::new(),
            type_by_pane: BTreeMap::new(),
            first_pane: pane,
        };
        v.pane_by_type.insert(PaneType::LeftPane, pane);
        v.type_by_pane.insert(pane, PaneType::LeftPane);
        let entity;
        if let Some(tab_model) = v.active() {
            entity = tab_model.active();
        } else {
            return v;
        }
        v.entity_by_pane.insert(v.focus, entity);
        v.entity_by_type.insert(PaneType::LeftPane, entity);
        v.pane_by_entity.insert(entity, v.focus);
        v.type_by_entity.insert(entity, PaneType::LeftPane);

        v
    }
    pub fn active(&self) -> Option<&TabModel> {
        self.panestates.get(self.focus)
    }
    pub fn active_mut(&mut self) -> Option<&mut TabModel> {
        self.panestates.get_mut(self.focus)
    }

    pub fn insert(&mut self, pane_type: PaneType, pane: pane_grid::Pane, split: pane_grid::Split) {
        if let Some(tab_model) = self.active_mut() {
            let title = match pane_type {
                PaneType::ButtonPane => "ButtonPane".to_string(),
                PaneType::TerminalPane => "TerminalPane".to_string(),
                PaneType::LeftPane => "LeftPane".to_string(),
                PaneType::RightPane => "RightPane".to_string(),
            };
            let entity = tab_model
                .insert()
                .text(title)
                //.closable()
                //.activate()
                .id();
            self.panes.push(pane);
            self.splits.push(split);
            self.focus = pane;
            self.panes_created += 1;
            self.pane_by_type.insert(pane_type, pane);
            self.type_by_pane.insert(pane, pane_type);
            self.entity_by_pane.insert(pane, entity);
            self.entity_by_type.insert(pane_type, entity);
            self.pane_by_entity.insert(entity, pane);
            self.type_by_entity.insert(entity, pane_type);
        }
    }

    pub fn _set_focus(&mut self, pane_type: PaneType) {
        if !self.pane_by_type.contains_key(&pane_type) {
            return;
        }
        let pane = self.pane_by_type[&pane_type];
        match pane_type {
            PaneType::ButtonPane => {
                let pane = self.pane_by_type[&PaneType::LeftPane];
                self.focus = pane;
            }
            PaneType::TerminalPane => self.focus = pane,
            PaneType::LeftPane => self.focus = pane,
            PaneType::RightPane => self.focus = pane,
        };
    }

    pub fn _focussed(&self) -> PaneType {
        return self.type_by_pane[&self.focus];
    }
}

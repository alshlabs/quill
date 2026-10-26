use gpui_kit::*;

use crate::icons::ICON_COMMENT;
use crate::icons::ICON_GEAR;
use crate::panels::panel_chat::PanelChat;
use crate::panels::panel_settings::PanelSettings;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TabId(u64);

impl TabId {
  pub fn element_id(
    self,
    name: &'static str,
  ) -> ElementId {
    (name, self.0).into()
  }
}

pub enum TabKind {
  Chat(Entity<PanelChat>),
  Settings(Entity<PanelSettings>),
}

impl TabKind {
  pub fn title(
    &self,
    cx: &App,
  ) -> SharedString {
    match self {
      TabKind::Chat(chat) => chat.read(cx).title(),
      TabKind::Settings(_) => "Settings".into(),
    }
  }

  pub fn icon(&self) -> &'static [u8] {
    match self {
      TabKind::Chat(_) => ICON_COMMENT,
      TabKind::Settings(_) => ICON_GEAR,
    }
  }

  pub fn is_dirty(
    &self,
    cx: &App,
  ) -> bool {
    match self {
      TabKind::Chat(_) => false,
      TabKind::Settings(settings) => settings.read(cx).is_dirty(),
    }
  }

  pub fn view(&self) -> AnyView {
    match self {
      TabKind::Chat(chat) => chat.clone().into(),
      TabKind::Settings(settings) => settings.clone().into(),
    }
  }
}

pub struct Tab {
  pub id: TabId,
  pub kind: TabKind,
}

#[derive(Default)]
pub struct Tabs {
  items: Vec<Tab>,
  active: Option<TabId>,
  next_id: u64,
}

impl Tabs {
  pub fn items(&self) -> &[Tab] {
    &self.items
  }

  pub fn active_id(&self) -> Option<TabId> {
    self.active
  }

  pub fn active(&self) -> Option<&Tab> {
    let active = self.active?;
    self.items.iter().find(|tab| tab.id == active)
  }

  pub fn open(
    &mut self,
    kind: TabKind,
    cx: &mut Context<Self>,
  ) -> TabId {
    let id = TabId(self.next_id);
    self.next_id += 1;
    self.items.push(Tab { id, kind });
    self.active = Some(id);
    cx.notify();
    id
  }

  pub fn activate(
    &mut self,
    id: TabId,
    cx: &mut Context<Self>,
  ) {
    if self.active == Some(id) || !self.items.iter().any(|tab| tab.id == id) {
      return;
    }
    self.active = Some(id);
    cx.notify();
  }

  pub fn activate_offset(
    &mut self,
    offset: isize,
    cx: &mut Context<Self>,
  ) {
    if self.items.is_empty() {
      return;
    }
    let current = self.index_of(self.active).unwrap_or(0) as isize;
    let next = (current + offset).rem_euclid(self.items.len() as isize) as usize;
    self.activate(self.items[next].id, cx);
  }

  pub fn close(
    &mut self,
    id: TabId,
    cx: &mut Context<Self>,
  ) {
    let Some(ix) = self.index_of(Some(id)) else {
      return;
    };
    self.items.remove(ix);
    if self.active == Some(id) {
      self.active = self.items.get(ix).or(self.items.last()).map(|tab| tab.id);
    }
    cx.notify();
  }

  fn index_of(
    &self,
    id: Option<TabId>,
  ) -> Option<usize> {
    let id = id?;
    self.items.iter().position(|tab| tab.id == id)
  }
}

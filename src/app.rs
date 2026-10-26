use gpui_kit::base::h_flex;
use gpui_kit::base::v_flex;
use gpui_kit::component::Icon;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

use crate::icons::ICON_BRUSH;
use crate::icons::ICON_CLOSE;
use crate::icons::ICON_EXTENSIONS;
use crate::icons::ICON_FOLDER_OPENED;
use crate::icons::ICON_GEAR;
use crate::icons::ICON_NEW_SESSION;
use crate::icons::ICON_SEARCH;
use crate::panels::about_blank::PanelAboutBlank;
use crate::panels::panel_chat::ChatEvent;
use crate::panels::panel_chat::PanelChat;
use crate::panels::panel_settings::DirtyChanged;
use crate::panels::panel_settings::PanelSettings;
use crate::services::updates;
use crate::sidebars::sidebar_chats::OpenChat;
use crate::sidebars::sidebar_chats::SidebarChats;
use crate::sidebars::sidebar_plugins::SidebarPlugins;
use crate::sidebars::sidebar_search::SidebarSearch;
use crate::sidebars::sidebar_workspaces::SidebarWorkspaces;
use crate::tabs::Tab;
use crate::tabs::TabKind;
use crate::tabs::Tabs;
use crate::theme;

const SIDEBAR_MARGIN: Pixels = px(4.0);
const SIDEBAR_MIN_WIDTH: Pixels = px(200.0);
const SIDEBAR_DEFAULT_WIDTH: Pixels = px(250.0);
const SIDEBAR_NAV_WIDTH: Pixels = px(42.0);
const TAB_BAR_HEIGHT: Pixels = px(32.0);
const TAB_GROUP: &str = "tab";

actions!(
  quill,
  [
    NewChat,
    OpenSettings,
    Save,
    CloseTab,
    NextTab,
    PrevTab,
    Update
  ]
);

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum UpdateState {
  #[default]
  UpToDate,
  Available,
}

impl Global for UpdateState {}

impl UpdateState {
  pub fn get(cx: &App) -> Self {
    cx.try_global::<Self>().copied().unwrap_or_default()
  }
}

pub fn init(cx: &mut App) {
  cx.bind_keys([
    KeyBinding::new("secondary-n", NewChat, None),
    KeyBinding::new("secondary-,", OpenSettings, None),
    KeyBinding::new("cmd-w", CloseTab, None),
    KeyBinding::new("secondary-s", Save, None),
    KeyBinding::new("ctrl-tab", NextTab, None),
    KeyBinding::new("ctrl-shift-tab", PrevTab, None),
  ]);
}

pub struct AppRoot {
  focus_handle: FocusHandle,
  tabs: Entity<Tabs>,
  sidebar_chats: Entity<SidebarChats>,
  _subscriptions: Vec<Subscription>,
  _update: Option<Task<()>>,
}

impl AppRoot {
  pub fn new(
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> Self {
    // Actions only reach `on_action` handlers on the focused element's path.
    let focus_handle = cx.focus_handle();
    window.focus(&focus_handle, cx);

    let tabs = cx.new(|_| Tabs::default());
    let sidebar_chats = cx.new(SidebarChats::new);
    let subscriptions = vec![
      cx.observe(&tabs, |_, _, cx| cx.notify()),
      cx.subscribe_in(
        &sidebar_chats,
        window,
        |this, _, event: &OpenChat, window, cx| this.open_chat(event.0.clone(), window, cx),
      ),
    ];

    let mut this = Self {
      focus_handle,
      tabs,
      sidebar_chats,
      _subscriptions: subscriptions,
      _update: None,
    };
    this.check_for_update(cx);
    this
  }

  fn check_for_update(
    &mut self,
    cx: &mut Context<Self>,
  ) {
    let check = updates::check_for_update();
    self._update = Some(cx.spawn(async move |this, cx| {
      let Ok(Some(_)) = check.await else {
        return;
      };
      let _ = this.update(cx, |_, cx| {
        cx.set_global(UpdateState::Available);
        cx.notify();
      });
    }));
  }

  fn install_update(
    &mut self,
    _: &Update,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    if UpdateState::get(cx) != UpdateState::Available {
      return;
    }

    // The process has to outlive its window while the update installs.
    cx.set_quit_mode(QuitMode::Explicit);
    window.remove_window();

    let upgrade = updates::try_upgrade();
    cx.spawn(async move |_, cx| {
      let _ = upgrade.await;
      // Relaunching even when the install failed brings the current release back
      // instead of leaving the app closed.
      let _ = updates::relaunch();
      cx.update(|cx| cx.quit());
    })
    .detach();
  }

  fn add_chat(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> Entity<PanelChat> {
    let chat = cx.new(|cx| PanelChat::new("New Chat", window, cx));
    cx.subscribe(&chat, |this, _, _: &ChatEvent, cx| {
      this
        .sidebar_chats
        .update(cx, |sidebar, cx| sidebar.refresh(cx));
      cx.notify();
    })
    .detach();
    self.tabs.update(cx, |tabs, cx| {
      tabs.open(TabKind::Chat(chat.clone()), cx);
    });
    chat
  }

  /// Switches to the tab already showing the session, otherwise opens it in a
  /// new tab.
  fn open_chat(
    &mut self,
    session_id: String,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    let open_tab = self
      .tabs
      .read(cx)
      .items()
      .iter()
      .find_map(|tab| match &tab.kind {
        TabKind::Chat(chat) => {
          (chat.read(cx).session_id() == Some(session_id.as_str())).then_some(tab.id)
        }
        TabKind::Settings(_) => None,
      });
    if let Some(id) = open_tab {
      self.tabs.update(cx, |tabs, cx| tabs.activate(id, cx));
      return;
    }

    let chat = self.add_chat(window, cx);
    chat.update(cx, |chat, cx| chat.load_session(session_id, cx));
  }

  fn new_chat(
    &mut self,
    _: &NewChat,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    self.add_chat(window, cx);
  }

  fn open_settings(
    &mut self,
    _: &OpenSettings,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    let open_tab = self
      .tabs
      .read(cx)
      .items()
      .iter()
      .find(|tab| matches!(tab.kind, TabKind::Settings(_)))
      .map(|tab| tab.id);
    if let Some(id) = open_tab {
      self.tabs.update(cx, |tabs, cx| tabs.activate(id, cx));
      return;
    }

    let settings = cx.new(|cx| PanelSettings::new(window, cx));
    cx.subscribe(&settings, |_, _, _: &DirtyChanged, cx| cx.notify())
      .detach();
    self.tabs.update(cx, |tabs, cx| {
      tabs.open(TabKind::Settings(settings), cx);
    });
  }

  fn close_tab(
    &mut self,
    _: &CloseTab,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    self.tabs.update(cx, |tabs, cx| {
      if let Some(id) = tabs.active_id() {
        tabs.close(id, cx);
      }
    });
  }

  fn next_tab(
    &mut self,
    _: &NextTab,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    self.tabs.update(cx, |tabs, cx| tabs.activate_offset(1, cx));
  }

  fn prev_tab(
    &mut self,
    _: &PrevTab,
    _window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    self
      .tabs
      .update(cx, |tabs, cx| tabs.activate_offset(-1, cx));
  }
}

impl Render for AppRoot {
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let sidebar_width =
      window.use_keyed_state("storybook-sidebar-width", cx, |_, _| None::<Pixels>);
    let width = *sidebar_width.read(cx);
    let sidebar_last_width =
      window.use_keyed_state("sidebar-last-width", cx, |_, _| SIDEBAR_DEFAULT_WIDTH);
    let dragging = window.use_keyed_state("storybook-sidebar-dragging", cx, |_, _| false);
    let hovered = window.use_keyed_state("storybook-dragbar-hovered", cx, |_, _| false);
    let sidebar_view = window.use_keyed_state("sidebar-view", cx, |_, _| None::<SidebarView>);
    let view = *sidebar_view.read(cx);

    v_flex()
      .track_focus(&self.focus_handle)
      .on_action(cx.listener(Self::new_chat))
      .on_action(cx.listener(Self::open_settings))
      .on_action(cx.listener(Self::close_tab))
      .on_action(cx.listener(Self::next_tab))
      .on_action(cx.listener(Self::prev_tab))
      .on_action(cx.listener(Self::install_update))
      .h_full()
      .w_full()
      // Title bar
      .child(div().flex_shrink_0().h(px(42.0)))
      // Side bar
      .child(
        h_flex()
          .id("storybook-body")
          .flex_1()
          .min_h_0()
          .w_full()
          .on_drag_move::<DraggedSidebar>({
            let sidebar_width = sidebar_width.clone();
            let sidebar_last_width = sidebar_last_width.clone();
            move |event, _, cx| {
              let width =
                event.event.position.x - event.bounds.left() - SIDEBAR_MARGIN - SIDEBAR_NAV_WIDTH;
              let width = (width >= SIDEBAR_MIN_WIDTH).then_some(width);

              if let Some(width) = width {
                sidebar_last_width.update(cx, |last_width, _| *last_width = width);
              }

              sidebar_width.update(cx, |sidebar_width, cx| {
                *sidebar_width = width;
                cx.notify();
              });
            }
          })
          .child(
            h_flex()
              .h_full()
              .border_1()
              .rounded(px(8.0))
              .bg(rgb(theme::colors::COLOR_PANEL_BACKGROUND))
              .border_color(rgb(theme::colors::COLOR_PANEL_BORDER))
              .flex_shrink_0()
              .ml(SIDEBAR_MARGIN)
              .child(sidebar_nav(
                width.and(view),
                sidebar_view,
                sidebar_width,
                sidebar_last_width,
              ))
              .children(width.map(|width| sidebar_contents(width, view, &self.sidebar_chats))),
          )
          .child(dragbar(dragging, hovered, cx))
          .child(main_content().child(tab_view(&self.tabs, cx))),
      )
      // Main body
      .child(div().flex_shrink_0().h(px(30.0)))
  }
}

struct DraggedSidebar;

#[derive(Clone, Copy, PartialEq)]
enum SidebarView {
  Chats,
  Workspaces,
  Search,
  Plugins,
}

fn sidebar_nav(
  view: Option<SidebarView>,
  sidebar_view: Entity<Option<SidebarView>>,
  sidebar_width: Entity<Option<Pixels>>,
  sidebar_last_width: Entity<Pixels>,
) -> Div {
  v_flex()
    .h_full()
    .flex_shrink_0()
    .justify_between()
    // Top of side nav
    .child(
      v_flex().items_center().w(SIDEBAR_NAV_WIDTH).children(
        [
          (SidebarView::Chats, ICON_NEW_SESSION),
          (SidebarView::Search, ICON_SEARCH),
          (SidebarView::Workspaces, ICON_FOLDER_OPENED),
          (SidebarView::Plugins, ICON_EXTENSIONS),
        ]
        .into_iter()
        .enumerate()
        .map(|(ix, (target, icon))| {
          let sidebar_view = sidebar_view.clone();
          let sidebar_width = sidebar_width.clone();
          let sidebar_last_width = sidebar_last_width.clone();
          let active = view == Some(target);
          sidebar_nav_icon(ix, active, icon).on_click(move |_, _, cx| {
            sidebar_view.update(cx, |view, cx| {
              *view = Some(target);
              cx.notify();
            });
            let last_width = *sidebar_last_width.read(cx);
            sidebar_width.update(cx, |width, cx| {
              *width = if active {
                None
              } else {
                width.or(Some(last_width))
              };
              cx.notify();
            });
          })
        }),
      ),
    )
    // Bottom of side nav
    .child(
      v_flex()
        .items_center()
        .w(SIDEBAR_NAV_WIDTH)
        .when(cfg!(debug_assertions), |this| {
          this.child(sidebar_nav_icon(4, false, ICON_BRUSH))
        })
        // .child(sidebar_nav_icon(5, false, ICON_ACCOUNT))
        .child(
          sidebar_nav_icon(6, false, ICON_GEAR)
            .on_click(|_, window, cx| window.dispatch_action(OpenSettings.boxed_clone(), cx)),
        ),
    )
}

fn sidebar_nav_icon(
  ix: usize,
  active: bool,
  icon: &[u8],
) -> Stateful<Div> {
  v_flex()
    .id(("sidebar-nav-icon", ix))
    .items_center()
    .justify_center()
    .w(px(32.0))
    .h(px(32.0))
    .mt(px(6.0))
    .mb(px(6.0))
    .text_color(rgb(theme::colors::COLOR_ICON_FOREGROUND))
    .rounded(px(4.0))
    .cursor_pointer()
    .when(active, |this| {
      this
        .bg(rgb(theme::colors::COLOR_LIST_SELECTION_BACKGROUND))
        .text_color(rgb(theme::colors::COLOR_ACTIVITY_BAR_FOREGROUND))
    })
    .hover(|this| this.bg(rgba(theme::colors::COLOR_TOOLBAR_HOVER_BACKGROUND_A)))
    .child(
      Icon::empty().data(icon).h(px(24.0)).w(px(24.0)), // .h_full().w_full()
    )
}

fn sidebar_contents(
  width: Pixels,
  view: Option<SidebarView>,
  sidebar_chats: &Entity<SidebarChats>,
) -> Div {
  div()
    .h_full()
    .w(width)
    .flex_shrink_0()
    .border_color(rgb(theme::colors::COLOR_PANEL_BORDER))
    .border_l_1()
    .map(|this| match view {
      Some(SidebarView::Chats) => this.child(sidebar_chats.clone()),
      Some(SidebarView::Workspaces) => this.child(SidebarWorkspaces::new()),
      Some(SidebarView::Search) => this.child(SidebarSearch::new()),
      Some(SidebarView::Plugins) => this.child(SidebarPlugins::new()),
      None => this,
    })
}

fn dragbar(
  dragging: Entity<bool>,
  hovered: Entity<bool>,
  cx: &App,
) -> impl IntoElement {
  let highlighted = *dragging.read(cx) || *hovered.read(cx);
  let color = rgb(theme::colors::COLOR_FOCUS_BORDER);
  let transparent = Rgba { a: 0.0, ..color };

  div()
    .id("dragbar")
    .h_full()
    .w(px(4.0))
    .flex_shrink_0()
    .rounded(px(8.0))
    .cursor_col_resize()
    .on_drag(DraggedSidebar, {
      let dragging = dragging.clone();
      move |_, _, _, cx| {
        set_flag(&dragging, true, cx);
        cx.new(|_| EmptyView)
      }
    })
    // The pointer may have left the dragbar by the time the drag ends.
    .on_mouse_up(MouseButton::Left, {
      let dragging = dragging.clone();
      move |_, _, cx| set_flag(&dragging, false, cx)
    })
    .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
      set_flag(&dragging, false, cx)
    })
    .on_hover(move |is_hovered, _, cx| set_flag(&hovered, *is_hovered, cx))
    .with_spring(
      "dragbar-highlight",
      SpringAnimation::new(SpringConfig::new(900.0, 60.0, 1.0)).to(highlighted),
      move |this, phase| this.bg(phase.interpolate_clamped(transparent, color)),
    )
}

fn set_flag(
  flag: &Entity<bool>,
  value: bool,
  cx: &mut App,
) {
  flag.update(cx, |flag, cx| {
    if *flag != value {
      *flag = value;
      cx.notify();
    }
  });
}

fn tab_view(
  tabs: &Entity<Tabs>,
  cx: &App,
) -> AnyElement {
  let Some(active) = tabs.read(cx).active() else {
    return PanelAboutBlank::new("about-blank").into_any_element();
  };

  v_flex()
    .size_full()
    .child(tab_bar(tabs, cx))
    .child(div().flex_1().min_h_0().child(active.kind.view()))
    .into_any_element()
}

fn tab_bar(
  tabs: &Entity<Tabs>,
  cx: &App,
) -> Div {
  let state = tabs.read(cx);
  let active = state.active_id();
  let items = state.items();

  h_flex()
    .relative()
    .h(TAB_BAR_HEIGHT)
    .flex_shrink_0()
    .rounded_t(px(8.0))
    .bg(rgb(theme::colors::COLOR_TAB_BAR_BACKGROUND))
    // .px(px(4.0))
    // Must stay the first child: tabs paint over it, so the active tab's
    // background hides the line beneath it.
    .child(
      div()
        .absolute()
        .bottom_0()
        .left_0()
        .right_0()
        .h(px(1.0))
        .bg(rgb(theme::colors::COLOR_EDITOR_BORDER)),
    )
    .children(items.iter().enumerate().flat_map(|(ix, tab)| {
      let separator = (ix > 0).then(|| {
        let visible = active != Some(items[ix - 1].id) && active != Some(tab.id);
        tab_separator(visible).into_any_element()
      });
      let item = tab_bar_item(tabs, tab, active == Some(tab.id), ix == 0, cx).into_any_element();
      separator.into_iter().chain([item])
    }))
    .child(
      div()
        .id("tab-bar-empty")
        .flex_1()
        .h_full()
        .on_click(|event, window, cx| {
          if event.click_count() == 2 {
            window.dispatch_action(NewChat.boxed_clone(), cx);
          }
        }),
    )
}

// Always rendered so tabs don't shift sideways when the active tab changes.
fn tab_separator(visible: bool) -> Div {
  div()
    .w(px(1.0))
    .h(px(16.0))
    .flex_shrink_0()
    .when(visible, |this| {
      this.bg(rgb(theme::colors::COLOR_EDITOR_BORDER))
    })
}

fn tab_bar_item(
  tabs: &Entity<Tabs>,
  tab: &Tab,
  active: bool,
  first: bool,
  cx: &App,
) -> Stateful<Div> {
  let id = tab.id;
  let dirty = tab.kind.is_dirty(cx);

  h_flex()
    .id(id.element_id("tab"))
    .group(TAB_GROUP)
    .h_full()
    .pl(px(12.0))
    .pr(px(6.0))
    .gap(px(8.0))
    .items_center()
    .rounded_t(px(8.0))
    .cursor_pointer()
    .text_xs()
    .text_color(rgb(theme::colors::COLOR_TAB_INACTIVE_FOREGROUND))
    // Every tab carries the border so selecting one doesn't shift its layout;
    // only the active tab makes it visible.
    .when(!first, |this| this.border_l_1())
    .border_r_1()
    .border_color(transparent_black())
    .map(|this| {
      if active {
        this
          .bg(rgb(theme::colors::COLOR_EDITOR_BACKGROUND))
          .text_color(rgb(theme::colors::COLOR_TAB_ACTIVE_FOREGROUND))
          .border_color(rgb(theme::colors::COLOR_EDITOR_BORDER))
      } else {
        this.hover(|this| this.text_color(rgb(theme::colors::COLOR_FOREGROUND)))
      }
    })
    .on_click({
      let tabs = tabs.clone();
      move |_, _, cx| tabs.update(cx, |tabs, cx| tabs.activate(id, cx))
    })
    .on_aux_click({
      let tabs = tabs.clone();
      move |event, _, cx| {
        if event.is_middle_click() {
          tabs.update(cx, |tabs, cx| tabs.close(id, cx));
        }
      }
    })
    .child(Icon::empty().data(tab.kind.icon()).h(px(12.0)).w(px(12.0)))
    .child(tab.kind.title(cx))
    .child(
      div()
        .id(id.element_id("tab-close"))
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .w(px(18.0))
        .h(px(18.0))
        .rounded(px(4.0))
        .hover(|this| this.bg(rgba(theme::colors::COLOR_TOOLBAR_HOVER_BACKGROUND_A)))
        .on_click({
          let tabs = tabs.clone();
          move |_, _, cx| {
            cx.stop_propagation();
            tabs.update(cx, |tabs, cx| tabs.close(id, cx));
          }
        })
        .map(|this| {
          let close = Icon::empty().data(ICON_CLOSE).h(px(10.0)).w(px(10.0));
          if !dirty {
            return this.child(close);
          }
          let dot_color = if active {
            theme::colors::COLOR_TAB_ACTIVE_FOREGROUND
          } else {
            theme::colors::COLOR_TAB_INACTIVE_FOREGROUND
          };
          this
            .child(
              div()
                .size(px(8.0))
                .rounded_full()
                .bg(rgb(dot_color))
                .group_hover(TAB_GROUP, |this| this.invisible()),
            )
            .child(
              div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .invisible()
                .group_hover(TAB_GROUP, |this| this.visible())
                .child(close),
            )
        }),
    )
}

fn main_content() -> Div {
  div()
    .relative()
    .h_full()
    .w_full()
    // .ml(px(4.0))
    .mr(px(4.0))
    .border_1()
    .bg(rgb(theme::colors::COLOR_EDITOR_BACKGROUND))
    .border_color(rgb(theme::colors::COLOR_EDITOR_BORDER))
    .rounded(px(8.0))
}

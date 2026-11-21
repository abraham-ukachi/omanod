slint::include_modules!();

use omanod_core::{
  catalog, list_models, provider, stream, Config, Message, ModelInfo, ProviderConfig, Role,
  Rgb, Store, StreamEvent, Theme as OmaTheme, Thread, Usage,
};
use slint::{Color, ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

#[derive(Clone)]
struct AppState {
  config: Config,
  store: Rc<Store>,
  active_thread: Option<String>,
  current_response: String,
  streaming: bool,
  models: Vec<ModelInfo>,
  active_provider: String,
  active_model: String,
  theme_source: String,
}

enum UiEvent {
  Delta(String),
  Done(Usage),
  Failed(String),
  RefreshModels(Vec<ModelInfo>),
}

#[derive(Clone)]
struct UiCtx {
  state: Rc<RefCell<AppState>>,
  messages: Rc<VecModel<UiMessage>>,
  threads: Rc<VecModel<UiThread>>,
  providers: Rc<VecModel<UiProvider>>,
  models: Rc<VecModel<UiModel>>,
  tx: Sender<UiEvent>,
}

impl UiCtx {
  fn refresh_threads(&self) {
    let state = self.state.borrow();
    let threads = state.store.threads().unwrap_or_default();
    self.threads.set_vec(thread_rows(&threads));
  }

  fn refresh_providers(&self, active: &str) {
    self.providers.set_vec(provider_rows(&catalog(), active));
  }
}

fn color(rgb: Rgb) -> Color {
  Color::from_argb_encoded(rgb.argb())
}

fn apply_theme(ui: &AppWindow, theme: &OmaTheme) {
  let g = ui.global::<Theme>();
  g.set_name(theme.name.clone().into());
  g.set_bg(color(theme.background));
  g.set_fg(color(theme.foreground));
  g.set_accent(color(theme.accent));
  g.set_accent_text(color(theme.accent_text()));
  g.set_surface(color(theme.surface()));
  g.set_hover(color(theme.hover()));
  g.set_border(color(theme.border()));
  g.set_muted(color(theme.muted()));
}

fn message_rows(messages: &[Message]) -> Vec<UiMessage> {
  messages
    .iter()
    .map(|m| UiMessage {
      role: match m.role {
        Role::User => "user".into(),
        Role::Assistant => "assistant".into(),
        Role::System => "system".into(),
      },
      text: m.content.clone().into(),
      streaming: false,
    })
    .collect()
}

fn thread_rows(threads: &[Thread]) -> Vec<UiThread> {
  threads
    .iter()
    .map(|t| UiThread {
      id: t.id.clone().into(),
      title: t.title.clone().into(),
    })
    .collect()
}

fn provider_rows(providers: &[ProviderConfig], active: &str) -> Vec<UiProvider> {
  providers
    .iter()
    .map(|p| {
      let state = if p.kind == omanod_core::ProviderKind::Unsupported {
        "not wired"
      } else if p.id != "ollama" && p.api_key().is_empty() {
        "no key"
      } else {
        "ready"
      };
      UiProvider {
        id: p.id.clone().into(),
        name: p.name.clone().into(),
        state: state.into(),
        active: p.id == active,
      }
    })
    .collect()
}

fn model_rows(models: &[ModelInfo], active: &str) -> Vec<UiModel> {
  models
    .iter()
    .map(|m| UiModel {
      id: m.id.clone().into(),
      name: m.name.clone().into(),
      provider: m.provider.clone().into(),
      active: m.id == active,
    })
    .collect()
}

fn current_theme(config: &Config) -> OmaTheme {
  if config.theme == "Omarchy" {
    OmaTheme::from_omarchy_current().unwrap_or_else(|_| OmaTheme::named("Tokyo Night"))
  } else {
    OmaTheme::named(&config.theme)
  }
}

fn omarchy_source() -> String {
  let home = std::env::var("HOME").unwrap_or_else(|_| "/home/abrahamukachi".into());
  std::fs::read_to_string(format!(
    "{home}/.local/state/omarchy/current/theme/colors.toml"
  ))
  .unwrap_or_default()
}

fn fetch_models(config: ProviderConfig, base_url: Option<String>, tx: Sender<UiEvent>) {
  std::thread::spawn(move || {
    let mut config = config;
    if config.id == "compatible" {
      if let Some(url) = base_url {
        config.base_url = url;
      }
    }
    match list_models(&config) {
      Ok(models) => {
        let _ = tx.send(UiEvent::RefreshModels(models));
      }
      Err(error) => {
        let _ = tx.send(UiEvent::Failed(error.to_string()));
      }
    }
  });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let ui = AppWindow::new()?;
  let config = Config::load()?;
  let store = Store::open()?;
  let theme_source = omarchy_source();

  let mut names = vec!["Omarchy".to_string()];
  names.extend(OmaTheme::all().iter().map(|t| t.name.clone()));

  let active_provider = config.provider.clone();
  let active_model = config.model.clone();
  let model_label = if active_model.is_empty() {
    provider(&active_provider)
      .map(|p| p.name)
      .unwrap_or_else(|| active_provider.clone())
  } else {
    format!(
      "{} / {}",
      provider(&active_provider)
        .map(|p| p.name)
        .unwrap_or_else(|| active_provider.clone()),
      active_model
    )
  };

  let state = Rc::new(RefCell::new(AppState {
    config: config.clone(),
    store: Rc::new(store),
    active_thread: None,
    current_response: String::new(),
    streaming: false,
    models: Vec::new(),
    active_provider: active_provider.clone(),
    active_model: active_model.clone(),
    theme_source: theme_source.clone(),
  }));

  let messages = Rc::new(VecModel::from(Vec::<UiMessage>::new()));
  let threads = Rc::new(VecModel::from(thread_rows(&state.borrow().store.threads()?)));
  let providers = Rc::new(VecModel::from(provider_rows(
    &catalog(),
    &active_provider,
  )));
  let models = Rc::new(VecModel::from(Vec::<UiModel>::new()));
  let themes = Rc::new(VecModel::from(
    names.into_iter().map(SharedString::from).collect::<Vec<_>>(),
  ));

  ui.set_messages(ModelRc::from(messages.clone()));
  ui.set_threads(ModelRc::from(threads.clone()));
  ui.set_providers(ModelRc::from(providers.clone()));
  ui.set_models(ModelRc::from(models.clone()));
  ui.set_themes(ModelRc::from(themes));
  ui.set_model_label(model_label.into());
  ui.set_active_provider(active_provider.clone().into());
  ui.set_active_model(active_model.clone().into());
  ui.set_status("ready".into());
  apply_theme(&ui, &current_theme(&config));

  let (tx, rx): (Sender<UiEvent>, Receiver<UiEvent>) = channel();
  let ctx = UiCtx {
    state,
    messages,
    threads,
    providers,
    models,
    tx,
  };

  let weak = ui.as_weak();
  let ctx2 = ctx.clone();
  ui.on_new_chat(move || {
    let Some(ui) = weak.upgrade() else {
      return;
    };
    let mut state = ctx2.state.borrow_mut();
    state.active_thread = None;
    state.current_response.clear();
    state.streaming = false;
    ctx2.messages.set_vec(Vec::<UiMessage>::new());
    ui.set_draft("".into());
    ui.set_status("new conversation".into());
  });

  let weak = ui.as_weak();
  let ctx2 = ctx.clone();
  ui.on_select_thread(move |id| {
    let Some(ui) = weak.upgrade() else {
      return;
    };
    let id = id.to_string();
    let messages = {
      let state = ctx2.state.borrow();
      state.store.messages(&id)
    };
    match messages {
      Ok(messages) => {
        ctx2.messages.set_vec(message_rows(&messages));
        ui.set_active_thread(id.clone().into());
        ui.set_status("ready".into());
        let mut state = ctx2.state.borrow_mut();
        state.active_thread = Some(id);
        state.current_response.clear();
      }
      Err(error) => ui.set_status(error.to_string().into()),
    }
  });

  let weak = ui.as_weak();
  let ctx2 = ctx.clone();
  ui.on_delete_thread(move |id| {
    let Some(ui) = weak.upgrade() else {
      return;
    };
    let id = id.to_string();
    let result = {
      let mut state = ctx2.state.borrow_mut();
      let result = state.store.delete_thread(&id);
      if state.active_thread.as_deref() == Some(&id) {
        state.active_thread = None;
        state.current_response.clear();
      }
      result
    };
    if let Err(error) = result {
      ui.set_status(error.to_string().into());
      return;
    }
    ctx2.messages.set_vec(Vec::<UiMessage>::new());
    ctx2.refresh_threads();
    ui.set_status("conversation removed".into());
  });

  let weak = ui.as_weak();
  let ctx2 = ctx.clone();
  ui.on_send(move || {
    let Some(ui) = weak.upgrade() else {
      return;
    };
    let draft = ui.get_draft().to_string().trim().to_string();
    if draft.is_empty() {
      return;
    }
    if ctx2.state.borrow().streaming {
      ui.set_status("busy".into());
      return;
    }
    ui.set_draft("".into());

    let (thread_id, history) = {
      let mut state = ctx2.state.borrow_mut();
      let thread_id = match state.active_thread.clone() {
        Some(id) => id,
        None => {
          let title: String = draft.chars().take(42).collect();
          match state.store.create_thread(&title) {
            Ok(id) => {
              state.active_thread = Some(id.clone());
              id
            }
            Err(error) => {
              ui.set_status(error.to_string().into());
              return;
            }
          }
        }
      };
      if let Err(error) = state.store.add_message(&thread_id, Role::User, &draft) {
        ui.set_status(error.to_string().into());
        return;
      }
      state.current_response.clear();
      state.streaming = true;
      match state.store.messages(&thread_id) {
        Ok(history) => (thread_id, history),
        Err(error) => {
          state.streaming = false;
          ui.set_status(error.to_string().into());
          return;
        }
      }
    };

    ui.set_status("generating".into());
    ui.set_active_thread(thread_id.clone().into());

    let mut rows = message_rows(&history);
    rows.push(UiMessage {
      role: "assistant".into(),
      text: "".into(),
      streaming: true,
    });
    ctx2.messages.set_vec(rows);
    ctx2.refresh_threads();

    let (provider, model, base_url) = {
      let state = ctx2.state.borrow();
      (
        provider(&state.active_provider),
        state.active_model.clone(),
        state.config.compatible_base_url.clone(),
      )
    };
    let Some(mut provider) = provider else {
      ctx2.state.borrow_mut().streaming = false;
      ui.set_status("no model provider selected".into());
      return;
    };
    if provider.id == "compatible" {
      if let Some(url) = base_url {
        provider.base_url = url;
      }
    }

    let tx = ctx2.tx.clone();
    std::thread::spawn(move || {
      let sink = |event| {
        let event = match event {
          StreamEvent::Delta(text) => UiEvent::Delta(text),
          StreamEvent::Done(usage) => UiEvent::Done(usage),
          StreamEvent::Error(error) => UiEvent::Failed(error),
        };
        let _ = tx.send(event);
      };
      if let Err(error) = stream(&provider, &model, &history, &sink) {
        let _ = tx.send(UiEvent::Failed(error.to_string()));
      }
    });
  });

  let ctx2 = ctx.clone();
  ui.on_select_provider(move |id| {
    let id = id.to_string();
    let (provider, base_url) = {
      let state = ctx2.state.borrow();
      (provider(&id), state.config.compatible_base_url.clone())
    };
    if let Some(provider) = provider {
      fetch_models(provider, base_url, ctx2.tx.clone());
    }
  });

  let weak = ui.as_weak();
  let ctx2 = ctx.clone();
  ui.on_select_model(move |id| {
    let Some(ui) = weak.upgrade() else {
      return;
    };
    let id = id.to_string();
    let model = {
      let state = ctx2.state.borrow();
      state.models.iter().find(|m| m.id == id).cloned()
    };
    let Some(model) = model else {
      return;
    };
    {
      let mut state = ctx2.state.borrow_mut();
      state.active_provider = model.provider.clone();
      state.active_model = model.id.clone();
      state.config.provider = model.provider.clone();
      state.config.model = model.id.clone();
      if let Err(error) = state.config.save() {
        ui.set_status(error.to_string().into());
        return;
      }
    }
    let provider_name = provider(&model.provider)
      .map(|p| p.name)
      .unwrap_or_else(|| model.provider.clone());
    ui.set_model_label(format!("{} / {}", provider_name, model.id).into());
    ui.set_active_provider(model.provider.clone().into());
    ui.set_active_model(model.id.clone().into());
    ctx2.refresh_providers(&model.provider);
    ui.set_status("model selected".into());
  });

  let weak = ui.as_weak();
  let ctx2 = ctx.clone();
  ui.on_select_theme(move |name| {
    let Some(ui) = weak.upgrade() else {
      return;
    };
    let name = name.to_string();
    let theme = {
      let mut state = ctx2.state.borrow_mut();
      state.config.theme = name.clone();
      if let Err(error) = state.config.save() {
        ui.set_status(error.to_string().into());
        return;
      }
      if name == "Omarchy" {
        state.theme_source = omarchy_source();
        current_theme(&state.config)
      } else {
        OmaTheme::named(&name)
      }
    };
    apply_theme(&ui, &theme);
    ui.set_status(format!("theme: {}", theme.name).into());
  });

  let ctx2 = ctx.clone();
  ui.on_refresh_models(move || {
    let (provider, base_url) = {
      let state = ctx2.state.borrow();
      (
        provider(&state.active_provider),
        state.config.compatible_base_url.clone(),
      )
    };
    if let Some(provider) = provider {
      fetch_models(provider, base_url, ctx2.tx.clone());
    }
  });

  if let Some(provider) = provider(&active_provider) {
    fetch_models(provider, config.compatible_base_url.clone(), ctx.tx.clone());
  }

  let weak = ui.as_weak();
  let ctx2 = ctx.clone();
  let timer = slint::Timer::default();
  timer.start(slint::TimerMode::Repeated, Duration::from_millis(16), move || {
    while let Ok(event) = rx.try_recv() {
      match event {
        UiEvent::Delta(text) => {
          let index = ctx2.messages.row_count().saturating_sub(1);
          if let Some(mut row) = ctx2.messages.row_data(index) {
            row.text = format!("{}{}", row.text, text).into();
            ctx2.messages.set_row_data(index, row);
          }
          ctx2.state.borrow_mut().current_response.push_str(&text);
        }
        UiEvent::Done(usage) => {
          let index = ctx2.messages.row_count().saturating_sub(1);
          if let Some(mut row) = ctx2.messages.row_data(index) {
            row.streaming = false;
            ctx2.messages.set_row_data(index, row);
          }
          let mut state = ctx2.state.borrow_mut();
          let response = state.current_response.clone();
          state.streaming = false;
          if let (Some(id), false) = (state.active_thread.clone(), response.is_empty()) {
            let _ = state.store.add_message(&id, Role::Assistant, &response);
          }
          drop(state);
          ctx2.refresh_threads();
          if let Some(ui) = weak.upgrade() {
            ui.set_status(format!("ready · {}↑ {}↓", usage.input, usage.output).into());
          }
        }
        UiEvent::Failed(error) => {
          let index = ctx2.messages.row_count().saturating_sub(1);
          if let Some(mut row) = ctx2.messages.row_data(index) {
            if row.streaming {
              if row.text.is_empty() {
                row.text = format!("error: {}", error).into();
              }
              row.streaming = false;
              ctx2.messages.set_row_data(index, row);
            }
          }
          ctx2.state.borrow_mut().streaming = false;
          if let Some(ui) = weak.upgrade() {
            ui.set_status(format!("error: {}", error).into());
          }
        }
        UiEvent::RefreshModels(found) => {
          let active = ctx2.state.borrow().active_model.clone();
          ctx2.state.borrow_mut().models = found.clone();
          ctx2.models.set_vec(model_rows(&found, &active));
        }
      }
    }
  });

  let weak = ui.as_weak();
  let state = ctx.state.clone();
  let theme_timer = slint::Timer::default();
  theme_timer.start(slint::TimerMode::Repeated, Duration::from_secs(1), move || {
    let source = omarchy_source();
    let mut state = state.borrow_mut();
    if state.config.theme == "Omarchy" && source != state.theme_source {
      state.theme_source = source.clone();
      if let Ok(theme) = OmaTheme::from_omarchy_colors("Omarchy", &source) {
        drop(state);
        if let Some(ui) = weak.upgrade() {
          apply_theme(&ui, &theme);
        }
      }
    }
  });

  ui.run()?;
  Ok(())
}

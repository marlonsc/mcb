//! Browse page — vector store collection viewer with keyboard navigation.

use super::{html_escape, html_page};
use crate::state::McbState;
use axum::extract::Extension;
use loco_rs::prelude::*;
use mcb_domain::value_objects::CollectionId;

/// Browse page handler.
///
/// # Errors
///
/// Fails when collection data cannot be loaded.
pub async fn browse_page(Extension(state): Extension<McbState>) -> Result<Response> {
    let collections = state
        .vector_store
        .list_collections()
        .await
        .map_err(|e| loco_rs::Error::string(&e.to_string()))?;

    let mut chunks: Vec<mcb_domain::value_objects::SearchResult> = Vec::new();
    for col in &collections {
        let id = CollectionId::from_string(&col.name);
        let vecs = state
            .vector_store
            .list_vectors(&id, mcb_utils::constants::DEFAULT_BROWSE_LIMIT)
            .await
            .map_err(|e| loco_rs::Error::string(&e.to_string()))?;
        chunks.extend(vecs);
    }

    let grid = if chunks.is_empty() {
        r#"<p class="no-chunks">No collections indexed yet. Use <code>index_repo</code> to index a codebase.</p>"#.to_owned()
    } else {
        chunks
            .iter()
            .enumerate()
            .map(|(i, c)| chunk_card(i, c))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let body = format!(
        r#"<h1>Browse</h1>
<div id="collections-grid" class="collections-grid">{grid}</div>
<script>{KEYBOARD_NAV_JS}</script>"#
    );
    html_page!("Browse", body)
}

/// Render a single code chunk card.
fn chunk_card(idx: usize, c: &mcb_domain::value_objects::SearchResult) -> String {
    let lang = html_escape(&c.language.to_lowercase());
    let content = html_escape(&c.content);
    let file = html_escape(&c.file_path);
    let id = html_escape(&c.id);
    format!(
        r#"<div class="code-chunk" data-chunk-id="{id}" data-index="{idx}" data-language="{lang}" tabindex="0">
  <div class="chunk-header">
    <span class="chunk-file">{file}</span>
    <span class="chunk-lang">{lang}</span>
    <span class="chunk-lines">:{line}</span>
  </div>
  <pre class="chunk-content"><code>{content}</code></pre>
</div>"#,
        line = c.start_line,
    )
}

/// Vim-style keyboard navigation (j/k/g/G/End/c/Esc).
const KEYBOARD_NAV_JS: &str = r#"(function(){
  function q(s){return Array.from(document.querySelectorAll(s));}
  function ai(ch){var a=document.querySelector('[data-active="true"]');
    if(a)return parseInt(a.dataset.index||'0',10);
    var f=document.activeElement;
    if(f&&f.hasAttribute('data-chunk-id'))return parseInt(f.dataset.index||'0',10);
    return -1;}
  function sa(ch,i){ch.forEach(function(c){c.removeAttribute('data-active');});
    var t=ch[i];if(t){t.setAttribute('data-active','true');t.focus();
    t.scrollIntoView({behavior:'smooth',block:'nearest'});}}
  var pg=false;
  document.addEventListener('keydown',function(e){
    var ch=q('[data-chunk-id]');if(!ch.length)return;
    var c=ai(ch);if(c<0)c=0;
    if(e.key==='j'){e.preventDefault();sa(ch,Math.min(c+1,ch.length-1));pg=false;}
    else if(e.key==='k'){e.preventDefault();sa(ch,Math.max(c-1,0));pg=false;}
    else if(e.key==='g'&&!e.shiftKey){e.preventDefault();
      if(pg){sa(ch,0);pg=false;}else{pg=true;setTimeout(function(){pg=false;},500);}}
    else if(e.key==='G'||(e.shiftKey&&(e.key==='g'||e.key==='G'))){e.preventDefault();sa(ch,ch.length-1);pg=false;}
    else if(e.key==='End'){e.preventDefault();sa(ch,ch.length-1);pg=false;}
    else if(e.key==='c'){e.preventDefault();var x=ch[c>=0?c:0];
      if(x){navigator.clipboard.writeText((x.textContent||'').trim()).catch(function(){});}pg=false;}
    else if(e.key==='Escape'){e.preventDefault();var a=document.querySelector('[data-active="true"]');
      if(a)a.removeAttribute('data-active');pg=false;}
    else{pg=false;}
  });
})()"#;

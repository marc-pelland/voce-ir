const e="hero-section",t="A landing-page hero with headline, subtitle, and CTA",o="A hero section with a large headline, subtitle text below it, and a primary call-to-action button. The button should have a hover animation and be keyboard accessible. Dark background, light text, coral accent for the button.",n={schema_version_major:0,schema_version_minor:1,root:{node_id:"root",document_language:"en",metadata:{title:"Hero Section Example"},semantic_nodes:[{node_id:"sem-main",role:"main",label:"Hero section"},{node_id:"sem-cta",role:"button",label:"Get started",tab_index:0}],children:[{value_type:"Container",value:{node_id:"hero",layout:"Stack",direction:"Column",main_align:"Center",cross_align:"Center",background:{r:12,g:12,b:14,a:255},semantic_node_id:"sem-main",children:[{value_type:"TextNode",value:{node_id:"headline",content:"Build extraordinary interfaces through conversation",font_size:{value:48,unit:"Px"},font_weight:"Bold",heading_level:1,text_align:"Center",color:{r:232,g:230,b:225,a:255}}},{value_type:"TextNode",value:{node_id:"subtitle",content:"AI generates typed binary IR. A compiler emits optimized output. No framework runtime.",font_size:{value:18,unit:"Px"},text_align:"Center",color:{r:155,g:154,b:148,a:255}}},{value_type:"Surface",value:{node_id:"cta-button",fill:{r:232,g:89,b:60,a:255},corner_radius:{top_left:{value:8,unit:"Px"},top_right:{value:8,unit:"Px"},bottom_right:{value:8,unit:"Px"},bottom_left:{value:8,unit:"Px"}},padding:{top:{value:12,unit:"Px"},right:{value:24,unit:"Px"},bottom:{value:12,unit:"Px"},left:{value:24,unit:"Px"}},semantic_node_id:"sem-cta",children:[{value_type:"TextNode",value:{node_id:"cta-label",content:"Get Started",font_size:{value:16,unit:"Px"},font_weight:"SemiBold",color:{r:255,g:255,b:255,a:255}}}]}},{value_type:"GestureHandler",value:{node_id:"cta-tap",target_node_id:"cta-button",gesture_type:"Tap",trigger_event:"click",keyboard_key:"Enter"}},{value_type:"AnimationTransition",value:{node_id:"cta-hover-anim",target_node_id:"cta-button",properties:[{property:"transform.scale",from:"1",to:"1.05"}],duration:{ms:200},reduced_motion:{strategy:"Remove"}}}]}}]}},r={diagnostics:[],errors:0,valid:!0,warnings:0},i=`<!DOCTYPE html>
<html lang="en" dir="ltr">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' https: data:">
<meta http-equiv="X-Content-Type-Options" content="nosniff">
<meta http-equiv="X-Frame-Options" content="DENY">
<meta name="referrer" content="strict-origin-when-cross-origin">
<title>Hero Section Example</title>
<style>
:root{--voce-fg:#111;--voce-bg:#fff;--voce-muted-fg:#666;--voce-border:rgba(127,127,127,.25);--voce-surface:rgba(127,127,127,.04);--voce-primary:#6366f1;--voce-primary-hover:#818cf8;--voce-error:#ef4444;--voce-warning:#f59e0b;--voce-success:#10b981}
@media (prefers-color-scheme:dark){:root{--voce-fg:#e8e8ec;--voce-bg:#0a0a0c;--voce-muted-fg:#8b8b94;--voce-border:rgba(255,255,255,.12);--voce-surface:rgba(255,255,255,.04)}}
*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;line-height:1.5;background:var(--voce-bg);color:var(--voce-fg)}
img{max-width:100%;height:auto;display:block}
h1,h2,h3,h4,h5,h6{line-height:1.2;margin-bottom:.5em}
p{line-height:1.6;margin-bottom:1em}
p:last-child,li:last-child{margin-bottom:0}
ul,ol{margin:0 0 1em 1.5em;padding-left:.5em}
li{line-height:1.6;margin-bottom:.25em}
code{font-family:ui-monospace,'SF Mono',Menlo,Consolas,monospace;font-size:.92em;background:var(--voce-surface);padding:.15em .35em;border-radius:4px}
pre{background:var(--voce-surface);border:1px solid var(--voce-border);border-radius:6px;padding:12px 14px;overflow-x:auto;margin:0 0 1em}
pre code{background:transparent;padding:0;border-radius:0;font-size:.95em}
blockquote{margin:0 0 1em;padding:.25em 1em;border-left:3px solid var(--voce-primary);color:var(--voce-muted-fg);font-style:italic}
hr{border:none;border-top:1px solid var(--voce-border);margin:1.5em 0}
table{border-collapse:collapse;width:100%;margin-bottom:1em}
th,td{padding:8px 12px;text-align:left;border-bottom:1px solid var(--voce-border)}
th{font-weight:600}
tbody tr:nth-child(even){background:var(--voce-surface)}
a{transition:opacity .15s}
a:hover{opacity:.8}
a:focus-visible{outline:2px solid var(--voce-primary,#6366f1);outline-offset:2px;border-radius:2px}
input,textarea,select{transition:border-color .15s,box-shadow .15s}
input:focus,textarea:focus,select:focus{outline:none;border-color:var(--voce-primary,#6366f1);box-shadow:0 0 0 3px rgba(99,102,241,.2)}
button,[role="button"]{cursor:pointer}
.voce-btn{cursor:pointer;transition:opacity .15s,transform .1s}
.voce-btn:hover{opacity:.9}
.voce-btn:active{transform:scale(.98)}
.voce-btn:focus-visible{outline:2px solid var(--voce-primary,#6366f1);outline-offset:2px;border-radius:2px}
form{display:flex;flex-direction:column;gap:14px;max-width:520px;width:100%}
form label{font-size:14px;font-weight:500;display:block}
input,textarea,select{font:inherit;color:inherit;background:rgba(127,127,127,.06);border:1px solid rgba(127,127,127,.25);border-radius:6px;padding:10px 12px;width:100%;min-height:44px}
textarea{min-height:120px;resize:vertical;font-family:inherit}
form button[type="submit"],form button:not([type]){font:inherit;font-weight:600;background:var(--voce-primary,#6366f1);color:#fff;border:none;border-radius:6px;padding:12px 20px;min-height:44px;align-self:flex-start;cursor:pointer;transition:opacity .15s,transform .1s}
form button[type="submit"]:hover,form button:not([type]):hover{opacity:.92}
form button[type="submit"]:active,form button:not([type]):active{transform:scale(.98)}
form button[type="submit"]:focus-visible,form button:not([type]):focus-visible{outline:2px solid var(--voce-primary,#6366f1);outline-offset:2px}
form [role="alert"]{font-size:13px;color:var(--voce-error,#ef4444)}
@media(max-width:520px){form{max-width:100%}form button[type="submit"],form button:not([type]){width:100%;align-self:stretch}}
[data-voce-id="cta-button"]{transition:scale 200ms ease;}
@media(prefers-reduced-motion:reduce){
[data-voce-id="cta-button"]{transition:none!important;}
}
</style>
</head>
<body>
  <main style="background-color:rgb(12,12,14);display:flex;flex-direction:column;justify-content:center;align-items:center;" role="main" aria-label="Hero section">
    <h1 style="color:rgb(232,230,225);font-size:48px;font-weight:700;text-align:center;">Build extraordinary interfaces through conversation</h1>
    <p style="color:rgb(155,154,148);font-size:18px;text-align:center;">AI generates typed binary IR. A compiler emits optimized output. No framework runtime.</p>
    <div style="background-color:rgb(232,89,60);border-radius:8px 8px 8px 8px;padding:12px 24px 12px 24px;" role="button" aria-label="Get started" tabindex="0" data-voce-id="cta-button">
      <p style="color:rgb(255,255,255);font-size:16px;font-weight:600;">Get Started</p>
    </div>
  </main>
<script>
document.addEventListener('DOMContentLoaded',()=>{
  const el_cta_tap=document.querySelector('[data-voce-id="cta-button"]');
  if(el_cta_tap){}
});
<\/script>
</body>
</html>
`,a=5139,l={id:e,label:t,prompt:o,ir:n,validation:r,html:i,sizeBytes:a};export{l as default,i as html,e as id,n as ir,t as label,o as prompt,a as sizeBytes,r as validation};

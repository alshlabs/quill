# Quill

## Performance Obsessed LLM Harness

Running Claude or Codex desktop, the applications are sluggish and heavy. Codex uses _1 gigabyte_ of ram for just the UI. This is due to the fact that they ship a full browser along with their application code and their application code is written in JavaScript. 

This technical choice has historically been practical for cross-platform development because developing seperate dedicated native applications using native technologies was an expensive process.

But we are in the age of AI and, by Grayskull, there's no excuse to ship low quality applications.

## Introducing Quill

<div align="center">
<img src="./.docs/screenshot-2026-20-25.png" />
<div align="center"><i>Look familiar?</i></div>
</div>

<br>

This is an LLM harness that uses a miniscule **40mb of ram** and runs at the framerate of your monitor (e.g. 144fps).

<div align="center">
<img src="./.docs/usage-2026-20-25.png" />
</div>

## But, How?

Quill uses a tiny game engine to render UI widgets. This allows it to leverage APIs like Metal, Vulkan and DirectX while also being absolutely tiny.

Naturally, game engines are crazy optimized, so the UI runs butter smooth and uses pretty much no resources.

Quill leverages the work done by the [Zed editor](https://zed.dev/), utilizing their [gpui-kit](https://crates.io/crates/gpui-kit). It's super easy to use, basically HTML and CSS.

## But, Why?

Have you seen the price of ram? I had to sell my Pokemon card collection just to get an extra 4gb of ram in my MacBook Pro... and it's still not enough.

I sort of view this as something of a case study on the fact that we can, in fact, develop high quality cross-platform applications today and it's actually not that hard.

Also, these days, software has never been worse to use. Encarpification, call it what you will, but it's just really fun to make something super high quality.

## Support The Development

Donate with Wise:

[<img height="200px" src=".docs/donate.png" >](https://wise.com/pay/me/davidsama1)
<script lang="ts">
import "@fontsource/noto-sans-jp";
</script>
<nav>

<div>
<img src="/jessielogo.png" alt="jessie logo">

</div>
</nav>
<main>
<h1>About</h1>
<p><strong>Jessie</strong> is a Rust UI framework. Our goal is to create a "hassle-free" development experience for making cross platform apps.</p>
<h2>Planned Features</h2>
<ul>
<li>Rendering with WGPU</li>
<li>Configuration with PKL</li>
<li>GTK 4 theme compatibility</li>
<li>Basic CSS support</li>
  </ul>

<p>Think this site looks like dogsh*t? We think so too! Please contribute to our <a href="https://github.com/jessie-framework/jessie">Github</a>, or join our <a href="https://discord.gg/GFBUKTH3XT">Discord</a>
</main>

<footer>
</footer>

<style>
img {
  height : 90%;
  
  object-fit: contain;
}
nav {
  position : fixed;
  width: 100%;
  height : 10vh;
  background-color : #222222;
}

div {
  height : 100%;
  display : flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
}

main {
  color : white;
  background: linear-gradient(#222222,#111111);
  flex: 1;
  display : flex;
  flex-direction: column;
  padding-top: 10vh;
  align-items: center;
}

.page {
  padding-top : 10vh;
  background-color : #111111;
}

footer {
  background-color: black;
  height : 100vh;
}

:global(body) {
  flex-direction:column ;
  align-content: center;
}

:global(html,body) {
  height : 100%;
  margin : 0%;
  font-family: "Noto Sans JP";
}
</style>

<svelte:body/>

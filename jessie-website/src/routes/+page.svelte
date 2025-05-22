<nav>
<div>
<img src="/jessielogo.png" alt="jessie logo">

</div>
</nav>
<div class="page">
<h1>About</h1>
<p>Placeholder</p>
</div>

<footer></footer>

<style>
img {
  height : 90%;
  
  object-fit: contain;
}
nav {
  position : fixed;
  width: 100%;
  height : 10vh;
  background-color : black;
}

div {
  height : 100%;
  display : flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
}

.page {
  padding-top : 10vh;
}

:global(body) {
  margin : 0px;
}
</style>

<svelte:body/>

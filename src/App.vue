<script setup lang="ts">
import { onMounted, ref, onBeforeUnmount } from "vue";
import { debounce } from 'lodash';

const canvasRef = ref<HTMLCanvasElement | null>(null);
function updateCanvas() {
  if (!canvasRef.value){
    return;
  }

  canvasRef.value.width = gridCols * cellSize;
  canvasRef.value.height = gridRows * cellSize;
  drawGrid();
}


const cellSize = 30;  // Cell size in pixels
let gridCols= 0;
let gridRows = 0;
let gridPadding = 50;
const onResize = debounce(() => {
  gridCols = Math.floor((window.innerWidth - gridPadding) / cellSize);
  gridRows = Math.floor((window.innerHeight - gridPadding) / cellSize);
  console.log(gridCols, gridRows);
  updateCanvas();
}, 250);

function drawGrid() {
  const ctx = canvasRef.value!.getContext('2d');
  if (!ctx) {
    return;
  }

  for (let row = 0; row < gridRows; row++) {
    for (let col = 0; col < gridCols; col++) {
      const blueGradient = Math.floor(Math.random() * 150 + 105);
      const greenGradient = blueGradient * 2 / 3;
      ctx.fillStyle = `rgb(${0}, ${greenGradient}, ${blueGradient})`;
      ctx.fillRect(col * cellSize, row * cellSize, cellSize, cellSize);
    }
  }
}

onMounted(() => {
  onResize();
  updateCanvas();
  window.addEventListener('resize', onResize);
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', onResize);
});
</script>

<template>
  <canvas ref="canvasRef"></canvas>
</template>

<style scoped lang="scss">
canvas {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);

  // Scss supports comments
}
</style>

<style>
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;

  font-family: monospace;
}

:root {
  overflow: hidden;

  background-color: #023;
}
</style>
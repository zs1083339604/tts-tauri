<script setup>
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {RouterView, useRouter} from 'vue-router'
import {
  Headset,
  Tools
} from '@element-plus/icons-vue'
import { ElMessage, ElMessageBox, ElLoading } from 'element-plus';
import {getCurrentWindow} from '@tauri-apps/api/window'
import {connect} from './utils/sqlite';
import { useVoicesStore } from "./store/voices";
import { show_error } from "./utils/function";
import mitter from "./utils/mitt";
import { useOptionStore } from "./store/option";

const loading = ElLoading.service({
  lock: true,
  text: '加载数据中……',
  background: 'rgba(0, 0, 0, 0.7)',
})

const timer = setInterval(() => {
  if(readObject.voices && readObject.option && readObject.exePath){
    clearInterval(timer);
    loading.close()
  }
}, 100);

const readObject = reactive({
  voices: false,
  option: false,
  exePath: false
})

// 初始化数据库
const voicesStore = useVoicesStore();
const optionStore = useOptionStore();

connect().then(()=>{
  // 获取配音员
  voicesStore.getVoicesList().then(()=>{
    readObject.voices = true;
  }).catch((error)=>{
    show_error(error);
  });

  // 获取app路径和设置对象
  optionStore.getExePath().then(()=>{
    readObject.exePath = true;
    return optionStore.getOption();
  }).then(()=>{
    readObject.option = true;
  }).catch((error)=> {
    show_error(error);
    readObject.exePath = true;
    readObject.option = true;
  })

}).catch((error)=>{
  ElMessageBox.alert(error, '初始化失败', {
    confirmButtonText: 'OK',
    callback: (action) => {
      const window = getCurrentWindow();
      window.close().catch(error=>{
        ElMessage.error(error);
      })
    },
  });
})

const router = useRouter();

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsg.value = await invoke("greet", { name: name.value });
}

const handleOpen = (key, keyPath) => {
  router.push({path: "/"  + key})
}

</script>

<template>
  <div class="appBox">
    <el-menu
      default-active="tts"
      class="menu"
      @select="handleOpen"
    >
      <el-menu-item index="tts">
        <el-icon><Headset /></el-icon>
        <template #title>配音</template>
      </el-menu-item>
      <el-menu-item index="set">
        <el-icon><Tools /></el-icon>
        <template #title>设置</template>
      </el-menu-item>
    </el-menu>
    <RouterView v-if="readObject.voices && readObject.option && readObject.exePath" class="router-view"></RouterView>
  </div>
</template>

<style scoped>
.router-view{
    height: 100%;
    box-sizing: border-box;
    padding: 20px;
    flex-grow: 1;
  }

  .appBox{
    display: flex;
    height: 100%;
  }

  .menu{
    width: 200px;
  }

</style>
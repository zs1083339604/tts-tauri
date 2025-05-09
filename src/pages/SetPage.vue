<script setup lang="ts">
    import { reactive, toRaw } from 'vue';
    import gethub from '../assets/github.svg'
    import gitee from '../assets/gitee.png'
    import { useOptionStore } from '../store/option';
    import { open } from '@tauri-apps/plugin-dialog';
    import { open as openShell } from '@tauri-apps/plugin-shell';
    import { show_error } from '../utils/function';
    import { ElMessage } from 'element-plus';

    const optionStore = useOptionStore();
    const form = reactive({
        savePath: "",
        openFolders: true,
        saveTTSOptions: true
    })

    const onSubmit = ()=>{
        optionStore.updataSoftOption(toRaw(form)).then(()=>{
            ElMessage.success("保存成功")
        }).catch((error)=>{
            show_error(error);
        })
    }

    const selectFolder = async ()=>{
        const file = await open({
            multiple: false,
            directory: true,
            defaultPath: form.savePath
        });

        if(file != null){
            // 选择了文件夹
            form.savePath = file.replaceAll("\\", "/");
        }
    }

    const openBrowser = async (url)=>{
        await openShell(url);
    }

    Object.assign(form, optionStore.softOption);
</script>

<template>
    <div class="set-page-box">
        <el-form :model="form" label-width="110" style="max-width: 1000px">
            <el-form-item label="保存位置">
                <div class="form-item-flex">
                    <p style="margin: 0;">{{ form.savePath }}</p>
                    <el-button type="primary" style="margin-left: 20px;" @click="selectFolder">选择文件夹</el-button>
                </div>
            </el-form-item>
            <el-form-item label="保存TTS设置">
                <el-switch v-model="form.saveTTSOptions" inline-prompt active-text="是" inactive-text="否"/>
            </el-form-item>
            <el-form-item label="自动打开文件夹">
                <el-switch v-model="form.openFolders" inline-prompt active-text="是" inactive-text="否"/>
            </el-form-item>
            <el-form-item label="检查更新">
                <div class="form-item-flex">
                    <div class="open-source" @click="openBrowser('https://github.com/zs1083339604/tts-tauri.git')">
                        <img :src="gethub" alt="github">
                        <p>github</p>
                    </div>
                    <div class="open-source" @click="openBrowser('https://gitee.com/lieranhuasha/tts-tauri.git')">
                        <img :src="gitee" alt="gitee">
                        <p>gitee</p>
                    </div>
                </div>
            </el-form-item>
            <el-form-item label="当前版本">
                <!-- 插眼：软件版本，本来应该从Rust获取，懒得写了 -->
                <div class="form-item-flex">
                    <span style="margin-right: 20px;">0.2.0</span>
                    <span>2025-05-09 更新</span>
                </div>
            </el-form-item>
            <el-form-item>
                <el-button type="primary" @click="onSubmit">保存</el-button>
            </el-form-item>
        </el-form>
    </div>
</template>

<style scoped>
    .form-item-flex{
        display: flex;
        width: 100%;
    }
    
    .open-source{
        margin-right: 20px;
        display: flex;
        align-items: center;
        padding: 5px;
        border: 1px solid #ccc;
        border-radius: 4px;
        cursor: pointer;
    }

    .open-source img{
        width: 25px;
        height: 25px;
        margin-right: 5px;
    }

    .open-source p{
        margin: 0;
    }
</style>
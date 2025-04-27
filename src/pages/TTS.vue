<script setup lang="ts">
    import { ref, reactive, toRaw, onMounted, onUnmounted } from 'vue';
    import { useVoicesStore } from '../store/voices';
    import { ElMessage } from 'element-plus'
    import { Check, Close } from '@element-plus/icons-vue'
    import { invoke } from '@tauri-apps/api/core';
    import mitter from '../utils/mitt';
    import { show_error } from '../utils/function';
    import { useOptionStore } from '../store/option';

    const voicesStore = useVoicesStore();
    const optionStore = useOptionStore();

    const formData = reactive({
        text: "",
        language: "",
        voice: "",
        pitch: 0,
        rate: 0,
        volume: 0,
        gender_sub_marker: true,
        sub_marker_type: "mergeByPunctuation",
        merge_by_number_number: 1
    });

    const languageList = ref([]);
    const voicesList = ref([]);
    const sub_marker_typeBox = ref(true);
    const audioDom = ref(null);
    const tryPlayList = [];


    // 按钮Loading对象
    const buttonLoadingObject = reactive({
        updataVoices: false,
        startTTSBtn: false,
        tryPlayBtn: false
    });

    onMounted(()=>{
        // 监听mitt事件
        mitter.on("updataVoices", ()=>{
            buttonLoadingObject.updataVoices = true;
        })

        mitter.on("updataVoicesed", ()=>{
            buttonLoadingObject.updataVoices = false;
        })
    })

    onUnmounted(()=>{
        mitter.off("updataVoices");
        mitter.off("updataVoicesed");
    })

    const updataVoices = async ()=>{
        voicesStore.updataVoices().then(()=>{
            renderLanguage();
        }).catch((error)=>{
            show_error(error);
        })
    }

    const languageChange = (newValue)=>{
        randerVoices(newValue, true);
    }

    const subMarkerChange = (newValue)=>{
        sub_marker_typeBox.value = newValue;
    }

    const startTTS = ()=>{
        ttsFun(toRaw(formData)).then((result)=>{
            console.log(result);
            // 载入base64音频
            audioDom.value.src = "data:audio/mp3;base64," + result.data.audio;
            audioDom.value.play();
        }).catch((error)=>{
            console.log(error)
            show_error(error);
        }).finally(()=>{
            buttonLoadingObject.startTTSBtn = false;
            buttonLoadingObject.tryPlayBtn = false;
        })
    }

    // 试听功能，暂时放弃，有BUG
    const tryPlay = async ()=>{
        // 通过标点符号分隔句子进行配音
        if(formData.text == ""){
            ElMessage.warning("请输入要配音的内容");
            return;
        }

        buttonLoadingObject.tryPlayBtn = true;
        buttonLoadingObject.startTTSBtn = true;

        const rule = /[，。？！：【「《“\.\,\?\!\:\"\[\<\n]/;
        const sentenceList = formData.text.split(rule);
        // 清除无用标点或空值
        for(let i = 0; i < sentenceList.length; i++){
            if(sentenceList[i] == "" || rule.test(sentenceList[i])){
                sentenceList.splice(i, 1);
                i--;
            }
        }

        let firstPlay = true;
        let timer = setInterval(async ()=>{
            if(sentenceList.length == 0){
                // 如果数据配音完成，退出计时器
                clearInterval(timer);
                return;
            }

            // 插眼，先发送3个
            if(tryPlayList.length < 3){
                const sentence = sentenceList.shift();
                const result = await ttsFun({...toRaw(formData), text: sentence});
                if(!result){
                    // 出错了也退出计时器
                    clearInterval(timer);
                    return;
                }
                console.log(result);
                // 存储base64音频
                tryPlayList.push("data:audio/mp3;base64," + result.data.audio);
                // 如果是第一次
                if(firstPlay){
                    audioDom.value.src = tryPlayList[0];
                    audioDom.value.play();
                    firstPlay = false;
                }
            }
        }, 500);

        // 添加事件监听，判断是否播放完成
        audioDom.value.addEventListener("ended", ()=>{
            tryPlayList.shift();
            if(tryPlayList.length != 0){
                audioDom.value.src = tryPlayList[0];
                audioDom.value.play();
            }else{
                buttonLoadingObject.tryPlayBtn = false;
                buttonLoadingObject.startTTSBtn = false;
            }
        });
    }

    // tts配音函数
    function ttsFun(ttsOption){
        return new Promise(async (resolve, reject) => {

            if(optionStore.softOption.saveTTSOptions){
                // 保存tts的设置
                optionStore.updataTTSOption(ttsOption);
            }
            
            if(ttsOption.text == ""){
                reject("请输入要配音的内容");
                return;
            }

            buttonLoadingObject.startTTSBtn = true;
            buttonLoadingObject.tryPlayBtn = true;

            try {
                const result = await invoke("start_tts", {data: {
                    ...ttsOption,
                    // 额外参数
                    root_path: optionStore.softOption.savePath,
                    open_folders: optionStore.softOption.openFolders
                }});
                console.log(result);
                resolve(result);
            } catch (error) {
                reject(error);
            }
        })
        

        
        // const result = await window.electronAPI.startTTS(ttsOption);
        // if(result.code != 200){
        //     ElMessage.error(result.msg);
        //     buttonLoadingObject.startTTSBtn = false;
        //     buttonLoadingObject.tryPlayBtn = false;
        //     return false;
        // }

        // return result;
    }

    function renderLanguage(){
        languageList.value.length = 0;

        for (const [key, value] of Object.entries(voicesStore.voices)) {
            languageList.value.push({
                label: key,
                value: value
            })
        }

        randerVoices(formData.language);
    }

    function randerVoices(language, enforcement = false){
        voicesList.value.length = 0;
        for(let i = 0; i < languageList.value.length; i++){
            const element = languageList.value[i];
            if(element.label == language){
                element.value.forEach(voice => {
                    voicesList.value.push({
                        label: voice.simpleName + " ("+voice.Gender+")",
                        value: voice.Name
                    });
                });
                
                if(enforcement){
                    formData.voice = voicesList.value[0].value;
                }

                if(formData.voice == ""){
                    formData.voice = voicesList.value[0].value;
                }
                
                break;
            }
        }
    }

    // 更新tts选项
    Object.assign(formData, optionStore.ttsOption);
    sub_marker_typeBox.value = formData.gender_sub_marker;
    // 渲染
    renderLanguage();
</script>

<template>
    <div class="tts">
        <div class="tts-box">
            <el-input
                v-model="formData.text"
                placeholder="请输入要配音的内容"
                type="textarea"
                class="textarea"
            />
            <div class="tool-box">
                <div class="select-box tool-item">
                    <el-select
                    v-model="formData.language"
                    placeholder="请选择语言"
                    no-data-text="无数据"
                    size="large"
                    style="width: 240px; margin-right: 20px;"
                    @change="languageChange"
                    filterable
                    >
                        <el-option
                            v-for="item in languageList"
                            :key="item.label"
                            :label="item.label"
                            :value="item.label"
                        />
                    </el-select>
                    <el-select
                    v-model="formData.voice"
                    placeholder="请选择配音员"
                    no-data-text="无数据"
                    size="large"
                    style="width: 240px"
                    >
                        <el-option
                            v-for="item in voicesList"
                            :key="item.value"
                            :label="item.label"
                            :value="item.value"
                        />
                    </el-select>
                </div>
                <div class="slider-box tool-item have-text">
                    <p>音调：</p>
                    <el-slider v-model="formData.pitch" show-input :max="100" :min="-100"/>
                </div>
                <div class="slider-box tool-item have-text">
                    <p>语速：</p>
                    <el-slider v-model="formData.rate" show-input :max="100" :min="-100"/>
                </div>
                <div class="slider-box tool-item have-text">
                    <p>音量：</p>
                    <el-slider v-model="formData.volume" show-input :max="100" :min="-100"/>
                </div>
                <div class="subMarker-box have-text">
                    <p>生成字幕：</p>
                    <el-switch 
                    v-model="formData.gender_sub_marker" 
                    inline-prompt
                    :active-icon="Check"
                    :inactive-icon="Close"
                    @change="subMarkerChange"
                    />
                </div>
                <div class="subMarker-radio-box tool-item">
                    <el-radio-group v-model="formData.sub_marker_type" v-if="sub_marker_typeBox">
                        <el-radio :value="'mergeByPunctuation'">按标点分隔</el-radio>
                        <el-radio :value="'autoWord'">自动分词</el-radio>
                        <el-radio :value="'mergeByNumber'">按数字分词</el-radio>
                    </el-radio-group>
                    <el-input-number v-model="formData.merge_by_number_number" :min="1" :max="15" v-if="formData.sub_marker_type == 'mergeByNumber' && sub_marker_typeBox" class="mergeByNumber-input"/>
                </div>
                <div class="button-box">
                    <el-button type="primary" @click="startTTS" :loading="buttonLoadingObject.startTTSBtn">开始配音</el-button>
                    <el-button type="primary" plain @click="tryPlay" :loading="buttonLoadingObject.tryPlayBtn" v-if="false">试听</el-button>
                    <el-button type="info" @click="updataVoices" :loading="buttonLoadingObject.updataVoices">更新配音员</el-button>
                </div>
            </div>
        </div>

        <audio ref="audioDom" controls class="tts-audio"></audio>
    </div>
    
</template>

<style scoped>
    .tts-box{
        display: flex;
    }

    .tool-box{
        width: 500px;
        margin-left: 40px;
    }

    .select-box{
        display: flex;
    }

    .tool-item{
        margin-bottom: 20px;
    }

    .have-text{
        display: flex;
        align-items: center;
    }

    .have-text p{
        flex-shrink: 0;
        margin-right: 10px;
    }

    .subMarker-radio-box{
        display: flex;
        align-items: center;
    }

    .mergeByNumber-input{
        width: 100px;
        margin-left: 20px;
    }

    .tts-audio{
        margin-top: 20px;
        width: 100%;
    }
</style>

<style>
    .textarea textarea{
        height: 100%;
        resize: none;
    }
</style>
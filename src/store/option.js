import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { select, update } from "../utils/sqlite";
import mitter from "../utils/mitt"

export const useOptionStore = defineStore("option", {
    actions: {
        getExePath(){
            return new Promise(async (resolve, reject) => {
                try {
                    const result = await invoke("get_exe_path");
                    if(result.code == 200){
                        this.app_exe_path = result.data.path.replaceAll("\\", "/");
                        if(this.app_exe_path.endsWith("/")){
                            this.app_exe_path = this.app_exe_path.slice(0, -1);
                        }

                        resolve();
                    }else{
                        reject(result.msg);
                    }
                } catch (error) {
                    reject(error)
                }
            })
        },
        getOption(){
            return new Promise((resolve, reject) => {
                select("option", ["softOption", "ttsOption"], "id = ?", [1]).then((result) => {
                    if(result.rows[0].ttsOption == ""){
                        // 使用默认值
                        this.ttsOption = {
                            language: "中文（大陆）",
                            voice: "Microsoft Server Speech Text to Speech Voice (zh-CN, XiaoxiaoNeural)",
                            pitch: 0,
                            rate: 0,
                            volume: 0,
                            gender_sub_marker: true,
                            sub_marker_type: "mergeByPunctuation",
                            merge_by_number_number: 1
                        }
                    }else{
                        this.ttsOption = JSON.parse(result.rows[0].ttsOption);
                    }

                    if(result.rows[0].softOption == ""){
                        // 使用默认值
                        this.softOption = {
                            savePath: this.app_exe_path + "/output",
                            openFolders: true,
                            saveTTSOptions: true
                        }
                    }else{
                        this.softOption = JSON.parse(result.rows[0].softOption);
                    }
                    resolve();
                }).catch((error)=>{
                    reject(error);
                })
            })
        },
        updataTTSOption(data){
            return new Promise((resolve, reject) => {
                const obj = {...JSON.parse(JSON.stringify(data)), text: ""};
                update("option", {ttsOption: JSON.stringify(obj)}, "id = ?", [1]).then((result) => {
                    // 深拷贝对象，上次写代码直接用toRaw就出Bug了
                    this.ttsOption = obj;
                    resolve();
                }).catch((error)=>{
                    reject(error);
                })
            });
        },
        updataSoftOption(data){
            return new Promise((resolve, reject) => {
                const newObjString = JSON.stringify(data);
                
                update("option", {softOption: newObjString}, "id = ?", [1]).then((result) => {
                    this.softOption = JSON.parse(newObjString);
                    resolve();
                }).catch((error)=>{
                    reject(error);
                })
            });
        }
    },
    state() {
        return{
            app_exe_path: "",
            softOption: null,
            ttsOption: null
        }
    },
})

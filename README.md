# Tauri + Vue 3

使用Tauri开发的微软配音工具

## 注意事项

- 第一次使用Tauri发布公开软件，据说Tauri的兼容性不好，有问题请提交。
- 音频使用base64传输，文本过多时会有性能问题

## 已完成功能

- [x] 基本配音功能
- [x] 字幕生成功能
- [x] 配音界面的设置保存
- [x] 设置页面

## 后续计划

- [ ] 多角色配音
- [ ] 长文本切片（可提高性能）
- [ ] 试听朗读（可用于小说阅读）
- [ ] 自定义停顿间隔（暂时无法适应字幕）
- [ ] 多端适配（Mac,Linux,安卓）
- [ ] 解决base64引发的长音频性能问题

## 开发

### 克隆代码

```
git clone https://gitee.com/lieranhuasha/tts-tauri.git
或
git clone https://github.com/zs1083339604/tts-tauri.git
```

### 安装依赖

```
cd tts-tauri
npm install
```

### 运行

```
npm run tauri dev
```

### 注意事项

使用Tauri开发，需要完成前置条件：[Tauri前置条件](https://tauri.app/zh-cn/start/prerequisites/)
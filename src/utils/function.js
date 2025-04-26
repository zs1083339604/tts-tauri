import { ElMessage } from "element-plus";

function show_error(error){
    let error_str = typeof error == 'string' ? error : "";

    if(error_str == ""){
        // error是一个对象，判断是否有msg或message属性
        if(error.msg){
            error_str = error.msg;
        }else if(error.message){
            error_str = error.message;
        }
    }

    ElMessage.error({
        dangerouslyUseHTMLString: true,
        message: error_str,
    });
}

export {
    show_error
}
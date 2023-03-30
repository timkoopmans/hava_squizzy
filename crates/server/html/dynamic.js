// create a chart instance
var myChart = echarts.init(document.getElementById('chart'));

// define the chart options
var option = {
    title: {
        text: 'Ping Response Times'
    },
    tooltip: {
        trigger: 'axis'
    },
    xAxis: {
        type: 'category',
        boundaryGap: false,
        data: ['1', '2', '3', '4', '5', '6', '7', '8', '9', '10', '11']
    },
    yAxis: {
        type: 'value',
        axisLabel: {
            formatter: '{value} ms'
        }
    },
    series: [{
        data: [41, 34, 32, 35, 30, 38, 34, 28, 29, 34, 41],
        type: 'line',
        areaStyle: {}
    }]
};

// set the chart options
myChart.setOption(option);
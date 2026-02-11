
var story = "I had scarcely thought of the theater for some years, when Kean arrived in this country; and it was more from curiosity than from any other motive, that I went to see, for the first time, the great actor of the age. I was soon lost to the recollection of being in a theater, or looking upon a great display of the \"mimic art.\" The simplicity, earnestness, and sincerity of his acting made me forgetful of the fiction, and bore me away with the power of reality and truth. If this be acting, said I, as I returned home, I may as well make the theater my school, and henceforward study nature at second hand. \nHow can I describe one who is almost as full of beauties as nature itself,—who grows upon us the more we become acquainted with him, and makes us sensible that the first time we saw him in any part, however much he may have moved us, we had but a partial apprehension of the many excellences of his acting? We cease to consider it as a mere amusement. It is an intellectual feast; and he who goes to it with a disposition and capacity to relish it, will receive from it more nourishment for his mind, than he would be likely to do in many other ways in twice the time. Our faculties are opened and enlivened by it; our reflections and recollections are of an elevated kind; and the voice which is sounding in our ears, long after we have left him, creates an inward harmony which is for our good. \nKean, in truth, stands very much in that relation to other players whom we have seen, that Shakspeare does to other dramatists. One player is called classical; another makes fine points here, and another there; Kean makes more fine points than all of them together; but in him these are only little prominences, showing their bright heads above a beautifully undulated surface. A continual change is going on in him, partaking of the nature of the varying scenes he is passing through, and the many thoughts and feelings which are shifting within him.";

// VARIABLES
var blockedOut = false;
var faded = false;
var punctuation = /([—.,?!;:"'])/;

// FUNCTIONS
function displayStory(storytext) {
  var poem = '';
  var array = storytext.split(' ');
  array.forEach(function(word){
    poem += "<span class>" + word + " </span>" 
  });
  $(".story").html(poem);
}





// Update text of poem
function updatePoem () {
  $("div.poem").html($("span.marked").text());
}


function newStory (){
  var storytext = $(".textarea").val()
  $('.story').html(displayStory(storytext));
  $(".textarea").val('');
  return false;
}

displayStory(story);








// CLICK RESET: Remove all classes, clear poem
$(".reset").on('click',function(){
$("span").removeClass('blocked faded marked');
$(".poem").html('(Marks words above to populate your poem)');
});


// Clicking on Words
$("div.story").on("click",function (evt){
  if (this === evt.target || $(evt.target).hasClass('faded') ){
  // do nothing
  } else {
      $(evt.target).removeClass("blocked")
      $(evt.target).toggleClass("marked")
  }
    updatePoem();
});


// Block out all words not "marked"
$(".blockOut").on('click', function(){
  if(blockedOut){
    $("span:not('.marked')").removeClass("blocked");
    blockedOut = false;
  } else {
    $("span:not('.marked')").addClass("blocked");
    blockedOut = true;
  }
});

// 
$(".clearMarked").on('click', function(){
  $("span").removeClass("marked");
  $(".poem").html('(Marks words above to populate your poem)');
});




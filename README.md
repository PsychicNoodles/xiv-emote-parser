# xiv-emote-parser

A parser library that converts emote log messages from FFXIV into a sequence of either static or dynamic text. Any dynamic components are dependent on global variables in the log message context, such as player names and gender.

For example, the following log message text:

```
<Clickable(<If(Equal(ObjectParameter(1),ObjectParameter(2)))>you<Else/><If(PlayerParameter(7))><SheetEn(ObjStr,2,PlayerParameter(7),1,1)/><Else/>ObjectParameter(2)</If></If>)/> <If(Equal(ObjectParameter(1),ObjectParameter(2)))>console<Else/>consoles</If> <If(Equal(ObjectParameter(1),ObjectParameter(3)))><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>you</If><Else/><If(PlayerParameter(8))><SheetEn(ObjStr,2,PlayerParameter(8),1,1)/><Else/>ObjectParameter(3)</If></If>.
```

can be converted into `your eyes brim over with tears.`, given that the origin of the message is the player character.

## To-do

- [ ] support `de` and `fr`
  - [ ] handle additional function and tag types
- [ ] make a cli?
